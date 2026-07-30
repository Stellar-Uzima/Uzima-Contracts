#![no_std]

pub fn storage_layout() -> &'static [&'static str] {
    &[
        "timelock::cfg",
        "timelock::queue",
        "meta_tx_forwarder::owner",
        "meta_tx_forwarder::nonce",
        "meta_tx_forwarder::relayer",
        "meta_tx_forwarder::fee_collector",
        "meta_tx_forwarder::user_pub_key",
        "medical_records::records",
        "patient_consent_management::consent",
        "token_sale::sale_config",
        "token_sale::contributions",
        "audit::entries",
        "access_control::roles",
    ]
}