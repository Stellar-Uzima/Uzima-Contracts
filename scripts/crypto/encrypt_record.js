#!/usr/bin/env node
'use strict';

const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

function base64urlEncode(buf) {
  return Buffer.from(buf)
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/g, '');
}

function base64urlDecode(str) {
  let s = String(str).replace(/-/g, '+').replace(/_/g, '/');
  while (s.length % 4 !== 0) s += '=';
  return Buffer.from(s, 'base64');
}

function parseArgs(argv) {
  const out = {};
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (!a.startsWith('--')) continue;
    const key = a.slice(2);
    const next = argv[i + 1];
    if (next && !next.startsWith('--')) {
      out[key] = next;
      i++;
    } else {
      out[key] = true;
    }
  }
  return out;
}

function usage() {
  const msg = `
Usage:
  node scripts/crypto/encrypt_record.js --in PLAINTEXT.json --recipients RECIPIENTS.json [--out CIPHERTEXT.bin] [--ciphertext-ref REF]

RECIPIENTS.json format:
  [
    {
      "recipient": "G... (Stellar address)",
      "key_version": 1,
      "x25519_public_key_hex": "32-byte hex string",
      "pq_wrapped_key_b64": null
    }
  ]

Outputs JSON to stdout:
  - ciphertext_ref + ciphertext_hash_hex to store on-chain
  - per-recipient KeyEnvelope objects (wrapped_key_b64/hex)

Wrapped-key format (X25519 envelope, version 1):
  0:        u8   version (=1)
  1..32:    32B  ephemeral X25519 public key (raw)
  33..48:   16B  hkdf salt
  49..60:   12B  AES-GCM iv
  61..92:   32B  encrypted DEK (AES-256-GCM)
  93..108:  16B  AES-GCM tag
`;
  process.stdout.write(msg.trimStart());
}

function readJson(p) {
  return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function ensureHex32Bytes(hex, fieldName) {
  if (typeof hex !== 'string') throw new Error(`${fieldName} must be a hex string`);
  const clean = hex.startsWith('0x') ? hex.slice(2) : hex;
  if (!/^[0-9a-fA-F]+$/.test(clean)) throw new Error(`${fieldName} must be hex`);
  const buf = Buffer.from(clean, 'hex');
  if (buf.length !== 32) throw new Error(`${fieldName} must be 32 bytes (got ${buf.length})`);
  return buf;
}

function makeX25519PublicKeyFromRaw(raw32) {
  const jwk = { kty: 'OKP', crv: 'X25519', x: base64urlEncode(raw32) };
  return crypto.createPublicKey({ key: jwk, format: 'jwk' });
}

function exportRawX25519PublicKey(keyObject) {
  const jwk = keyObject.export({ format: 'jwk' });
  if (!jwk || jwk.kty !== 'OKP' || jwk.crv !== 'X25519' || !jwk.x) {
    throw new Error('Unexpected ephemeral key export; expected OKP/X25519 JWK with "x"');
  }
  return base64urlDecode(jwk.x);
}

function hkdfSha256(ikm, salt, info, len) {
  return crypto.hkdfSync('sha256', ikm, salt, Buffer.from(info, 'utf8'), len);
}

function aes256GcmEncrypt(key, plaintext) {
  const iv = crypto.randomBytes(12);
  const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
  const ciphertext = Buffer.concat([cipher.update(plaintext), cipher.final()]);
  const tag = cipher.getAuthTag();
  return { iv, ciphertext, tag };
}

function main() {
  const args = parseArgs(process.argv);
  if (args.help || args.h) {
    usage();
    process.exit(0);
  }

  if (!args.in || !args.recipients) {
    usage();
    process.exit(1);
  }

  const plaintextPath = String(args.in);
  const recipientsPath = String(args.recipients);

  const recipients = readJson(recipientsPath);
  if (!Array.isArray(recipients) || recipients.length === 0) {
    throw new Error('RECIPIENTS.json must be a non-empty array');
  }

  const plaintext = fs.readFileSync(plaintextPath);

  // 1) Encrypt record payload with a random DEK (AES-256-GCM).
  const dek = crypto.randomBytes(32);
  const payloadEnc = aes256GcmEncrypt(dek, plaintext);
  const ciphertextBlob = Buffer.concat([payloadEnc.iv, payloadEnc.ciphertext, payloadEnc.tag]);
  const ciphertextHash = crypto.createHash('sha256').update(ciphertextBlob).digest();

  const outPath =
    args.out
      ? String(args.out)
      : path.join(process.cwd(), `encrypted_record_${Date.now()}.bin`);
  fs.writeFileSync(outPath, ciphertextBlob);

  const ciphertextRef = args['ciphertext-ref'] ? String(args['ciphertext-ref']) : `file:${outPath}`;

  // 2) Wrap DEK for each recipient using ephemeral X25519 + HKDF-SHA256 + AES-256-GCM.
  const envelopes = recipients.map((r, idx) => {
    if (!r || typeof r !== 'object') throw new Error(`Recipient entry ${idx} must be an object`);
    if (typeof r.recipient !== 'string' || r.recipient.length === 0) {
      throw new Error(`Recipient entry ${idx} must include "recipient"`);
    }
    if (!Number.isInteger(r.key_version) || r.key_version <= 0) {
      throw new Error(`Recipient entry ${idx} must include positive integer "key_version"`);
    }
    const recipientPubRaw = ensureHex32Bytes(r.x25519_public_key_hex, `recipients[${idx}].x25519_public_key_hex`);
    const recipientPubKey = makeX25519PublicKeyFromRaw(recipientPubRaw);

    const eph = crypto.generateKeyPairSync('x25519');
    const ephPubRaw = exportRawX25519PublicKey(eph.publicKey);

    const shared = crypto.diffieHellman({ privateKey: eph.privateKey, publicKey: recipientPubKey });
    const salt = crypto.randomBytes(16);
    const wrapKey = hkdfSha256(shared, salt, 'uzima-e2e-envelope-v1', 32);
    const dekEnc = aes256GcmEncrypt(wrapKey, dek);

    const version = Buffer.from([1]);
    const wrappedKey = Buffer.concat([version, ephPubRaw, salt, dekEnc.iv, dekEnc.ciphertext, dekEnc.tag]);

    const pqWrappedKeyB64 = typeof r.pq_wrapped_key_b64 === 'string' && r.pq_wrapped_key_b64.length > 0 ? r.pq_wrapped_key_b64 : null;
    const algorithm = pqWrappedKeyB64 ? 'HybridX25519Kyber768' : 'X25519';

    return {
      recipient: r.recipient,
      key_version: r.key_version,
      algorithm,
      wrapped_key_b64: wrappedKey.toString('base64'),
      wrapped_key_hex: wrappedKey.toString('hex'),
      pq_wrapped_key_b64: pqWrappedKeyB64,
    };
  });

  const out = {
    ciphertext_file: outPath,
    ciphertext_ref: ciphertextRef,
    ciphertext_hash_hex: ciphertextHash.toString('hex'),
    envelopes,
  };

  process.stdout.write(JSON.stringify(out, null, 2) + '\n');
}

main();

