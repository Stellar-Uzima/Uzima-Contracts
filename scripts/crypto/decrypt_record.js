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
  node scripts/crypto/decrypt_record.js --ciphertext CIPHERTEXT.bin --envelope ENVELOPE.json --key-bundle KEY_BUNDLE.json [--out PLAINTEXT.out]

ENVELOPE.json:
  A single envelope object (one element from the encrypt_record output "envelopes" array).

KEY_BUNDLE.json:
  Output of generate_key_bundle.js with --include-private (must include encryption.private_jwk).
`;
  process.stdout.write(msg.trimStart());
}

function readJson(p) {
  return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function makeX25519PublicKeyFromRaw(raw32) {
  const jwk = { kty: 'OKP', crv: 'X25519', x: base64urlEncode(raw32) };
  return crypto.createPublicKey({ key: jwk, format: 'jwk' });
}

function hkdfSha256(ikm, salt, info, len) {
  return crypto.hkdfSync('sha256', ikm, salt, Buffer.from(info, 'utf8'), len);
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

  if (!args.ciphertext || !args.envelope || !args['key-bundle']) {
    usage();
    process.exit(1);
  }

  const ciphertextBlob = fs.readFileSync(String(args.ciphertext));
  if (ciphertextBlob.length < 12 + 16 + 1) {
    throw new Error('Ciphertext blob too small');
  }
  const payloadIv = ciphertextBlob.subarray(0, 12);
  const payloadTag = ciphertextBlob.subarray(ciphertextBlob.length - 16);
  const payloadCiphertext = ciphertextBlob.subarray(12, ciphertextBlob.length - 16);

  const envelope = readJson(String(args.envelope));
  if (!envelope || typeof envelope !== 'object') throw new Error('Invalid envelope JSON');
  if (typeof envelope.wrapped_key_b64 !== 'string') throw new Error('Envelope must include wrapped_key_b64');

  const keyBundle = readJson(String(args['key-bundle']));
  const privateJwk = keyBundle?.encryption?.private_jwk;
  if (!privateJwk || privateJwk.kty !== 'OKP' || privateJwk.crv !== 'X25519' || !privateJwk.d) {
    throw new Error('Key bundle must include encryption.private_jwk (OKP/X25519 with "d")');
  }
  const recipientPriv = crypto.createPrivateKey({ key: privateJwk, format: 'jwk' });

  const wrappedKey = Buffer.from(envelope.wrapped_key_b64, 'base64');
  if (wrappedKey.length !== 109 || wrappedKey[0] !== 1) {
    throw new Error(`Unsupported wrapped-key format; expected version=1 and length=109 (got ${wrappedKey.length})`);
  }
  const ephPubRaw = wrappedKey.subarray(1, 33);
  const salt = wrappedKey.subarray(33, 49);
  const wrapIv = wrappedKey.subarray(49, 61);
  const encDek = wrappedKey.subarray(61, 93);
  const wrapTag = wrappedKey.subarray(93, 109);

  const ephPubKey = makeX25519PublicKeyFromRaw(ephPubRaw);
  const shared = crypto.diffieHellman({ privateKey: recipientPriv, publicKey: ephPubKey });
  const wrapKey = hkdfSha256(shared, salt, 'uzima-e2e-envelope-v1', 32);
  const dek = aes256GcmDecrypt(wrapKey, wrapIv, encDek, wrapTag);
  if (dek.length !== 32) {
    throw new Error(`Unexpected DEK length (got ${dek.length})`);
  }

  const plaintext = aes256GcmDecrypt(dek, payloadIv, payloadCiphertext, payloadTag);

  if (args.out) {
    fs.writeFileSync(String(args.out), plaintext);
  } else {
    process.stdout.write(plaintext);
  }
}

main();

