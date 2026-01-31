#!/usr/bin/env node
'use strict';

const crypto = require('crypto');
const fs = require('fs');

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
  node scripts/crypto/rewrap_envelope.js \\
    --old-envelope OLD_ENVELOPE.json \\
    --old-key-bundle OLD_KEY_BUNDLE.json \\
    --new-recipient-x25519 NEW_RECIPIENT_PUBKEY_HEX \\
    --new-key-version N \\
    [--out NEW_ENVELOPE.json]

Rewraps (rotates) the symmetric DEK envelope after the recipient rotates their registry key bundle.

Notes:
  - Requires the old private key to decrypt the DEK.
  - Produces a new X25519 envelope (version=1 format) for the new public key + key_version.
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

function aes256GcmDecrypt(key, iv, ciphertext, tag) {
  const decipher = crypto.createDecipheriv('aes-256-gcm', key, iv);
  decipher.setAuthTag(tag);
  return Buffer.concat([decipher.update(ciphertext), decipher.final()]);
}

function main() {
  const args = parseArgs(process.argv);
  if (args.help || args.h) {
    usage();
    process.exit(0);
  }

  if (!args['old-envelope'] || !args['old-key-bundle'] || !args['new-recipient-x25519'] || !args['new-key-version']) {
    usage();
    process.exit(1);
  }

  const oldEnvelope = readJson(String(args['old-envelope']));
  if (!oldEnvelope || typeof oldEnvelope.wrapped_key_b64 !== 'string') {
    throw new Error('OLD_ENVELOPE.json must include wrapped_key_b64');
  }

  const keyBundle = readJson(String(args['old-key-bundle']));
  const privateJwk = keyBundle?.encryption?.private_jwk;
  if (!privateJwk || privateJwk.kty !== 'OKP' || privateJwk.crv !== 'X25519' || !privateJwk.d) {
    throw new Error('Old key bundle must include encryption.private_jwk (OKP/X25519 with "d")');
  }
  const oldPriv = crypto.createPrivateKey({ key: privateJwk, format: 'jwk' });

  const wrappedKey = Buffer.from(oldEnvelope.wrapped_key_b64, 'base64');
  if (wrappedKey.length !== 109 || wrappedKey[0] !== 1) {
    throw new Error(`Unsupported wrapped-key format; expected version=1 and length=109 (got ${wrappedKey.length})`);
  }
  const ephPubRaw = wrappedKey.subarray(1, 33);
  const salt = wrappedKey.subarray(33, 49);
  const wrapIv = wrappedKey.subarray(49, 61);
  const encDek = wrappedKey.subarray(61, 93);
  const wrapTag = wrappedKey.subarray(93, 109);

  const ephPubKey = makeX25519PublicKeyFromRaw(ephPubRaw);
  const shared = crypto.diffieHellman({ privateKey: oldPriv, publicKey: ephPubKey });
  const wrapKey = hkdfSha256(shared, salt, 'uzima-e2e-envelope-v1', 32);
  const dek = aes256GcmDecrypt(wrapKey, wrapIv, encDek, wrapTag);
  if (dek.length !== 32) {
    throw new Error(`Unexpected DEK length (got ${dek.length})`);
  }

  const newRecipientPubRaw = ensureHex32Bytes(String(args['new-recipient-x25519']), '--new-recipient-x25519');
  const newRecipientPub = makeX25519PublicKeyFromRaw(newRecipientPubRaw);
  const eph2 = crypto.generateKeyPairSync('x25519');
  const eph2PubRaw = exportRawX25519PublicKey(eph2.publicKey);
  const shared2 = crypto.diffieHellman({ privateKey: eph2.privateKey, publicKey: newRecipientPub });
  const salt2 = crypto.randomBytes(16);
  const wrapKey2 = hkdfSha256(shared2, salt2, 'uzima-e2e-envelope-v1', 32);
  const dekEnc2 = aes256GcmEncrypt(wrapKey2, dek);

  const version = Buffer.from([1]);
  const wrappedKey2 = Buffer.concat([version, eph2PubRaw, salt2, dekEnc2.iv, dekEnc2.ciphertext, dekEnc2.tag]);

  const newKeyVersion = Number(args['new-key-version']);
  if (!Number.isInteger(newKeyVersion) || newKeyVersion <= 0) {
    throw new Error('--new-key-version must be a positive integer');
  }

  const outEnvelope = {
    recipient: oldEnvelope.recipient ?? null,
    key_version: newKeyVersion,
    algorithm: 'X25519',
    wrapped_key_b64: wrappedKey2.toString('base64'),
    wrapped_key_hex: wrappedKey2.toString('hex'),
    pq_wrapped_key_b64: null,
  };

  const json = JSON.stringify(outEnvelope, null, 2) + '\n';
  if (args.out) {
    fs.writeFileSync(String(args.out), json, { encoding: 'utf8', flag: 'w' });
  } else {
    process.stdout.write(json);
  }
}

main();

