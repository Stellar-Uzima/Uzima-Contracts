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
  node scripts/crypto/generate_key_bundle.js [--out FILE] [--signing] [--include-private]

Outputs a JSON object with an X25519 encryption keypair (and optionally an Ed25519 signing keypair).

Notes:
  - Public key bytes are emitted as raw 32-byte values (hex) suitable for registering in \`crypto_registry\`.
  - Private keys are only included with --include-private. Do not commit private keys.
`;
  process.stdout.write(msg.trimStart());
}

function toRawOkpPublicHex(keyObject) {
  const jwk = keyObject.export({ format: 'jwk' });
  if (!jwk || jwk.kty !== 'OKP' || !jwk.x) {
    throw new Error('Unexpected key export; expected OKP JWK with "x"');
  }
  return base64urlDecode(jwk.x).toString('hex');
}

function main() {
  const args = parseArgs(process.argv);
  if (args.help || args.h) {
    usage();
    process.exit(0);
  }

  const includePrivate = Boolean(args['include-private']);
  const includeSigning = Boolean(args.signing);

  const encryption = crypto.generateKeyPairSync('x25519');
  const encryptionPublicHex = toRawOkpPublicHex(encryption.publicKey);

  const result = {
    generated_at: new Date().toISOString(),
    encryption: {
      algorithm: 'X25519',
      public_key_hex: encryptionPublicHex,
      public_jwk: encryption.publicKey.export({ format: 'jwk' }),
    },
  };

  if (includePrivate) {
    result.encryption.private_jwk = encryption.privateKey.export({ format: 'jwk' });
  }

  if (includeSigning) {
    const signing = crypto.generateKeyPairSync('ed25519');
    const signingPublicHex = toRawOkpPublicHex(signing.publicKey);
    result.signing = {
      algorithm: 'Ed25519',
      public_key_hex: signingPublicHex,
      public_jwk: signing.publicKey.export({ format: 'jwk' }),
    };
    if (includePrivate) {
      result.signing.private_jwk = signing.privateKey.export({ format: 'jwk' });
    }
  }

  // Convenience structure aligned to `contracts/crypto_registry`.
  result.crypto_registry_payload = {
    encryption_key: { algorithm: 'X25519', key_hex: encryptionPublicHex },
    pq_encryption_key: null,
    signing_key: includeSigning ? { algorithm: 'Ed25519', key_hex: result.signing.public_key_hex } : null,
  };

  const json = JSON.stringify(result, null, 2);
  if (args.out) {
    fs.writeFileSync(String(args.out), json + '\n', { encoding: 'utf8', flag: 'w' });
  } else {
    process.stdout.write(json + '\n');
  }
}

main();

