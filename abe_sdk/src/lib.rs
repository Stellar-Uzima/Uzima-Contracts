use rabe::error::RabeError;
use rabe::schemes::ac17::{
    cp_decrypt, cp_encrypt, cp_keygen, setup, Ac17CpCiphertext, Ac17CpSecretKey, Ac17MasterKey,
    Ac17PublicKey,
};
use rand::{thread_rng, Rng};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};
use thiserror::Error;

const SHARE_MODULUS: i64 = 257;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyNode {
    Attribute(AttributeRequirement),
    And(Vec<PolicyNode>),
    Or(Vec<PolicyNode>),
    Threshold {
        required: usize,
        children: Vec<PolicyNode>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeRequirement {
    pub namespace: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssuedAttribute {
    pub namespace: String,
    pub value: String,
    pub epoch: u32,
    pub expires_at_unix: Option<u64>,
    pub is_verified: bool,
    pub is_active: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AttributeDirectory {
    epochs: BTreeMap<(String, String), u32>,
    issued: Vec<IssuedAttribute>,
}

#[derive(Clone)]
pub struct AbeAuthority {
    pub public_key: Ac17PublicKey,
    pub master_key: Ac17MasterKey,
}

#[derive(Clone)]
pub enum EncryptedPolicyNode {
    Leaf {
        attr_token: String,
        ciphertext: Ac17CpCiphertext,
    },
    Branch {
        required: usize,
        children: Vec<EncryptedPolicyNode>,
    },
}

#[derive(Clone)]
pub struct EncryptedDekPackage {
    pub root: EncryptedPolicyNode,
    pub attribute_count: usize,
    pub revocation_epoch: u32,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub attribute_count: usize,
    pub decrypt_duration: Duration,
}

#[derive(Debug, Error)]
pub enum AbeSdkError {
    #[error("invalid policy: {0}")]
    InvalidPolicy(&'static str),
    #[error("policy not satisfied")]
    PolicyNotSatisfied,
    #[error("share decoding failed")]
    ShareDecoding,
    #[error("cp-abe error: {0}")]
    Crypto(#[from] RabeError),
}

impl AttributeRequirement {
    pub fn new(namespace: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            value: value.into(),
        }
    }
}

impl PolicyNode {
    pub fn attribute_count(&self) -> usize {
        match self {
            PolicyNode::Attribute(_) => 1,
            PolicyNode::And(children) | PolicyNode::Or(children) => {
                children.iter().map(Self::attribute_count).sum()
            }
            PolicyNode::Threshold { children, .. } => {
                children.iter().map(Self::attribute_count).sum()
            }
        }
    }
}

impl AttributeDirectory {
    pub fn issue(&mut self, attr: IssuedAttribute) {
        let key = (attr.namespace.clone(), attr.value.clone());
        self.epochs.entry(key).or_insert(1);
        self.issued.retain(|existing| {
            !(existing.namespace == attr.namespace && existing.value == attr.value)
        });
        self.issued.push(attr);
    }

    pub fn revoke(&mut self, namespace: &str, value: &str) {
        for attr in &mut self.issued {
            if attr.namespace == namespace && attr.value == value {
                attr.is_active = false;
            }
        }
        let entry = self
            .epochs
            .entry((namespace.to_string(), value.to_string()))
            .or_insert(1);
        *entry += 1;
    }

    pub fn current_epoch(&self, namespace: &str, value: &str) -> u32 {
        self.epochs
            .get(&(namespace.to_string(), value.to_string()))
            .copied()
            .unwrap_or(1)
    }

    pub fn active_tokens(&self, now_unix: u64) -> Vec<String> {
        self.issued
            .iter()
            .filter(|attr| attr.is_active)
            .filter(|attr| {
                attr.expires_at_unix
                    .map(|expiry| expiry > now_unix)
                    .unwrap_or(true)
            })
            .filter(|attr| {
                if attr.namespace == "location"
                    || attr.namespace == "region"
                    || attr.namespace == "facility"
                {
                    attr.is_verified
                } else {
                    true
                }
            })
            .filter(|attr| self.current_epoch(&attr.namespace, &attr.value) == attr.epoch)
            .map(|attr| canonical_attribute_token(&attr.namespace, &attr.value, attr.epoch))
            .collect()
    }
}

impl AbeAuthority {
    pub fn setup() -> Result<Self, AbeSdkError> {
        let (public_key, master_key) = setup();
        Ok(Self {
            public_key,
            master_key,
        })
    }

    pub fn issue_secret_key(
        &self,
        directory: &AttributeDirectory,
        now_unix: u64,
    ) -> Result<Ac17CpSecretKey, AbeSdkError> {
        let attrs = directory.active_tokens(now_unix);
        Ok(cp_keygen(&self.public_key, &self.master_key, &attrs)?)
    }
}

pub fn encrypt_dek_for_policy(
    authority: &AbeAuthority,
    policy: &PolicyNode,
    dek: &[u8],
    directory: &AttributeDirectory,
) -> Result<EncryptedDekPackage, AbeSdkError> {
    if dek.is_empty() {
        return Err(AbeSdkError::InvalidPolicy("dek must not be empty"));
    }
    validate_policy(policy)?;
    let root = encrypt_policy_node(authority, policy, dek, directory)?;
    Ok(EncryptedDekPackage {
        root,
        attribute_count: policy.attribute_count(),
        revocation_epoch: max_policy_epoch(policy, directory),
    })
}

pub fn decrypt_dek(
    secret_key: &Ac17CpSecretKey,
    package: &EncryptedDekPackage,
    directory: &AttributeDirectory,
    now_unix: u64,
) -> Result<Vec<u8>, AbeSdkError> {
    let tokens: BTreeSet<String> = directory.active_tokens(now_unix).into_iter().collect();
    decrypt_policy_node(secret_key, &package.root, &tokens)
}

pub fn benchmark_authorized_decryption(
    authority: &AbeAuthority,
    policy: &PolicyNode,
    directory: &AttributeDirectory,
    now_unix: u64,
) -> Result<BenchmarkResult, AbeSdkError> {
    let dek = vec![42u8; 32];
    let package = encrypt_dek_for_policy(authority, policy, &dek, directory)?;
    let secret_key = authority.issue_secret_key(directory, now_unix)?;
    let started = Instant::now();
    let decrypted = decrypt_dek(&secret_key, &package, directory, now_unix)?;
    let duration = started.elapsed();
    if decrypted != dek {
        return Err(AbeSdkError::PolicyNotSatisfied);
    }
    Ok(BenchmarkResult {
        attribute_count: package.attribute_count,
        decrypt_duration: duration,
    })
}

fn validate_policy(policy: &PolicyNode) -> Result<(), AbeSdkError> {
    match policy {
        PolicyNode::Attribute(attr) => {
            if attr.namespace.is_empty() || attr.value.is_empty() {
                return Err(AbeSdkError::InvalidPolicy("attributes must be non-empty"));
            }
            Ok(())
        }
        PolicyNode::And(children) | PolicyNode::Or(children) => {
            if children.is_empty() {
                return Err(AbeSdkError::InvalidPolicy(
                    "boolean node must have children",
                ));
            }
            for child in children {
                validate_policy(child)?;
            }
            Ok(())
        }
        PolicyNode::Threshold { required, children } => {
            if children.is_empty() || *required == 0 || *required > children.len() {
                return Err(AbeSdkError::InvalidPolicy("invalid threshold gate"));
            }
            for child in children {
                validate_policy(child)?;
            }
            Ok(())
        }
    }
}

fn max_policy_epoch(policy: &PolicyNode, directory: &AttributeDirectory) -> u32 {
    match policy {
        PolicyNode::Attribute(attr) => directory.current_epoch(&attr.namespace, &attr.value),
        PolicyNode::And(children) | PolicyNode::Or(children) => children
            .iter()
            .map(|child| max_policy_epoch(child, directory))
            .max()
            .unwrap_or(1),
        PolicyNode::Threshold { children, .. } => children
            .iter()
            .map(|child| max_policy_epoch(child, directory))
            .max()
            .unwrap_or(1),
    }
}

fn canonical_attribute_token(namespace: &str, value: &str, epoch: u32) -> String {
    format!("{namespace}:{value}@{epoch}")
}

fn encrypt_policy_node(
    authority: &AbeAuthority,
    policy: &PolicyNode,
    secret: &[u8],
    directory: &AttributeDirectory,
) -> Result<EncryptedPolicyNode, AbeSdkError> {
    match policy {
        PolicyNode::Attribute(attr) => {
            let token = canonical_attribute_token(
                &attr.namespace,
                &attr.value,
                directory.current_epoch(&attr.namespace, &attr.value),
            );
            let ciphertext = cp_encrypt(&authority.public_key, &token, secret)?;
            Ok(EncryptedPolicyNode::Leaf {
                attr_token: token,
                ciphertext,
            })
        }
        PolicyNode::And(children) => {
            encrypt_threshold_branch(authority, children, children.len(), secret, directory)
        }
        PolicyNode::Or(children) => {
            encrypt_threshold_branch(authority, children, 1, secret, directory)
        }
        PolicyNode::Threshold { required, children } => {
            encrypt_threshold_branch(authority, children, *required, secret, directory)
        }
    }
}

fn encrypt_threshold_branch(
    authority: &AbeAuthority,
    children: &[PolicyNode],
    required: usize,
    secret: &[u8],
    directory: &AttributeDirectory,
) -> Result<EncryptedPolicyNode, AbeSdkError> {
    let shares = split_secret(secret, required, children.len());
    let mut encrypted_children = Vec::with_capacity(children.len());
    for (index, child) in children.iter().enumerate() {
        encrypted_children.push(encrypt_policy_node(
            authority,
            child,
            &shares[index],
            directory,
        )?);
    }
    Ok(EncryptedPolicyNode::Branch {
        required,
        children: encrypted_children,
    })
}

fn decrypt_policy_node(
    secret_key: &Ac17CpSecretKey,
    node: &EncryptedPolicyNode,
    tokens: &BTreeSet<String>,
) -> Result<Vec<u8>, AbeSdkError> {
    match node {
        EncryptedPolicyNode::Leaf {
            attr_token,
            ciphertext,
        } => {
            if !tokens.contains(attr_token) {
                return Err(AbeSdkError::PolicyNotSatisfied);
            }
            Ok(cp_decrypt(secret_key, ciphertext)?)
        }
        EncryptedPolicyNode::Branch { required, children } => {
            let mut successful = Vec::new();
            for (index, child) in children.iter().enumerate() {
                if let Ok(share) = decrypt_policy_node(secret_key, child, tokens) {
                    successful.push((index + 1, share));
                    if successful.len() == *required {
                        break;
                    }
                }
            }
            if successful.len() < *required {
                return Err(AbeSdkError::PolicyNotSatisfied);
            }
            reconstruct_secret(&successful[..*required])
        }
    }
}

fn split_secret(secret: &[u8], threshold: usize, share_count: usize) -> Vec<Vec<u8>> {
    if threshold == 1 {
        return (0..share_count).map(|_| secret.to_vec()).collect();
    }

    let mut rng = thread_rng();
    let mut shares = vec![vec![0u8; secret.len() * 2]; share_count];

    for (byte_index, byte) in secret.iter().enumerate() {
        let mut coefficients = vec![*byte as i64];
        for _ in 1..threshold {
            coefficients.push(rng.gen_range(0..SHARE_MODULUS));
        }

        for x in 1..=share_count {
            let mut y = 0i64;
            let mut power = 1i64;
            for coefficient in &coefficients {
                y = (y + (coefficient * power)) % SHARE_MODULUS;
                power = (power * x as i64) % SHARE_MODULUS;
            }
            let encoded = y as u16;
            let offset = byte_index * 2;
            shares[x - 1][offset] = (encoded >> 8) as u8;
            shares[x - 1][offset + 1] = encoded as u8;
        }
    }

    shares
}

fn reconstruct_secret(shares: &[(usize, Vec<u8>)]) -> Result<Vec<u8>, AbeSdkError> {
    if shares.is_empty() {
        return Err(AbeSdkError::ShareDecoding);
    }
    let share_len = shares[0].1.len();
    if share_len % 2 != 0 {
        return Err(AbeSdkError::ShareDecoding);
    }

    let byte_len = share_len / 2;
    let mut secret = vec![0u8; byte_len];
    for byte_index in 0..byte_len {
        let mut points = Vec::with_capacity(shares.len());
        for (x, share) in shares {
            let offset = byte_index * 2;
            let y = u16::from_be_bytes([share[offset], share[offset + 1]]) as i64;
            points.push((*x as i64, y));
        }
        let recovered = lagrange_interpolate_zero(&points)?;
        if !(0..=255).contains(&recovered) {
            return Err(AbeSdkError::ShareDecoding);
        }
        secret[byte_index] = recovered as u8;
    }
    Ok(secret)
}

fn lagrange_interpolate_zero(points: &[(i64, i64)]) -> Result<i64, AbeSdkError> {
    let mut result = 0i64;
    for (i, (x_i, y_i)) in points.iter().enumerate() {
        let mut numerator = 1i64;
        let mut denominator = 1i64;
        for (j, (x_j, _)) in points.iter().enumerate() {
            if i == j {
                continue;
            }
            numerator = mod_field(numerator * -x_j);
            denominator = mod_field(denominator * (x_i - x_j));
        }
        let inv = mod_inverse(denominator).ok_or(AbeSdkError::ShareDecoding)?;
        result = mod_field(result + (y_i * numerator * inv));
    }
    Ok(mod_field(result))
}

fn mod_field(value: i64) -> i64 {
    let mut reduced = value % SHARE_MODULUS;
    if reduced < 0 {
        reduced += SHARE_MODULUS;
    }
    reduced
}

fn mod_inverse(value: i64) -> Option<i64> {
    let mut t = 0i64;
    let mut new_t = 1i64;
    let mut r = SHARE_MODULUS;
    let mut new_r = mod_field(value);

    while new_r != 0 {
        let quotient = r / new_r;
        (t, new_t) = (new_t, t - quotient * new_t);
        (r, new_r) = (new_r, r - quotient * new_r);
    }

    if r > 1 {
        return None;
    }
    Some(mod_field(t))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attribute(namespace: &str, value: &str) -> PolicyNode {
        PolicyNode::Attribute(AttributeRequirement::new(namespace, value))
    }

    fn make_directory(now: u64) -> AttributeDirectory {
        let mut directory = AttributeDirectory::default();
        for (namespace, value) in [
            ("role", "doctor"),
            ("department", "oncology"),
            ("permission", "read_confidential"),
            ("region", "KE"),
            ("facility", "uzima-main"),
            ("location", "nairobi"),
            ("valid_until", "2026-12-31"),
            ("project", "a"),
            ("project", "b"),
            ("project", "c"),
        ] {
            directory.issue(IssuedAttribute {
                namespace: namespace.to_string(),
                value: value.to_string(),
                epoch: 1,
                expires_at_unix: Some(now + 1_000),
                is_verified: true,
                is_active: true,
            });
        }
        directory
    }

    #[test]
    fn supports_and_or_and_threshold_policies() {
        let now = 1_700_000_000;
        let directory = make_directory(now);
        let authority = AbeAuthority::setup().expect("setup");
        let secret_key = authority.issue_secret_key(&directory, now).expect("keygen");

        let policies = [
            PolicyNode::And(vec![
                attribute("role", "doctor"),
                attribute("department", "oncology"),
            ]),
            PolicyNode::Or(vec![
                attribute("role", "doctor"),
                attribute("role", "emergency_responder"),
            ]),
            PolicyNode::Threshold {
                required: 2,
                children: vec![
                    attribute("project", "a"),
                    attribute("project", "b"),
                    attribute("project", "c"),
                ],
            },
        ];

        for policy in policies {
            let package = encrypt_dek_for_policy(&authority, &policy, b"uzima-secret", &directory)
                .expect("enc");
            let decrypted = decrypt_dek(&secret_key, &package, &directory, now).expect("dec");
            assert_eq!(decrypted, b"uzima-secret");
        }
    }

    #[test]
    fn supports_nested_time_and_location_attributes() {
        let now = 1_700_000_000;
        let directory = make_directory(now);
        let authority = AbeAuthority::setup().expect("setup");
        let secret_key = authority.issue_secret_key(&directory, now).expect("keygen");

        let policy = PolicyNode::And(vec![
            attribute("role", "doctor"),
            attribute("location", "nairobi"),
            attribute("valid_until", "2026-12-31"),
            PolicyNode::Threshold {
                required: 2,
                children: vec![
                    attribute("project", "a"),
                    attribute("project", "b"),
                    attribute("project", "c"),
                ],
            },
        ]);

        let package =
            encrypt_dek_for_policy(&authority, &policy, b"uzima-secret", &directory).expect("enc");
        let decrypted = decrypt_dek(&secret_key, &package, &directory, now).expect("dec");
        assert_eq!(decrypted, b"uzima-secret");
    }

    #[test]
    fn rejects_malformed_threshold_policy() {
        let invalid = PolicyNode::Threshold {
            required: 3,
            children: vec![
                attribute("role", "doctor"),
                attribute("department", "oncology"),
            ],
        };
        let authority = AbeAuthority::setup().expect("setup");
        let directory = make_directory(1_700_000_000);
        let result = encrypt_dek_for_policy(&authority, &invalid, b"secret", &directory);
        assert!(matches!(result, Err(AbeSdkError::InvalidPolicy(_))));
    }

    #[test]
    fn rejects_revoked_or_expired_attributes() {
        let now = 1_700_000_000;
        let mut directory = make_directory(now);
        directory.revoke("department", "oncology");

        let authority = AbeAuthority::setup().expect("setup");
        let stale_key = authority
            .issue_secret_key(&make_directory(now), now)
            .expect("keygen");
        let policy = PolicyNode::And(vec![
            attribute("role", "doctor"),
            attribute("department", "oncology"),
        ]);

        let package =
            encrypt_dek_for_policy(&authority, &policy, b"uzima-secret", &directory).expect("enc");
        let decrypted = decrypt_dek(&stale_key, &package, &directory, now);
        assert!(matches!(decrypted, Err(AbeSdkError::PolicyNotSatisfied)));
    }

    #[test]
    fn supports_large_policy_benchmark_harness() {
        let now = 1_700_000_000;
        let mut directory = AttributeDirectory::default();
        let mut children = Vec::new();
        for index in 0..120 {
            let value = format!("claim-{index}");
            directory.issue(IssuedAttribute {
                namespace: "project".to_string(),
                value: value.clone(),
                epoch: 1,
                expires_at_unix: Some(now + 5_000),
                is_verified: true,
                is_active: true,
            });
            children.push(attribute("project", &value));
        }

        let policy = PolicyNode::Threshold {
            required: 60,
            children,
        };
        let authority = AbeAuthority::setup().expect("setup");
        let result =
            benchmark_authorized_decryption(&authority, &policy, &directory, now).expect("bench");
        assert_eq!(result.attribute_count, 120);
        println!(
            "authorized CP-ABE decrypt benchmark: {:?} for {} attributes",
            result.decrypt_duration, result.attribute_count
        );
    }
}
