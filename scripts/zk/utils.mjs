#!/usr/bin/env node

import crypto from "node:crypto";

function normalize(value) {
  if (Array.isArray(value)) {
    return value.map(normalize);
  }
  if (value && typeof value === "object") {
    const out = {};
    for (const key of Object.keys(value).sort()) {
      out[key] = normalize(value[key]);
    }
    return out;
  }
  return value;
}

export function canonicalJson(value) {
  return JSON.stringify(normalize(value));
}

export function sha256Hex(input) {
  const hasher = crypto.createHash("sha256");
  if (Buffer.isBuffer(input)) {
    hasher.update(input);
  } else if (typeof input === "string") {
    hasher.update(Buffer.from(input, "utf8"));
  } else {
    hasher.update(Buffer.from(canonicalJson(input), "utf8"));
  }
  return hasher.digest("hex");
}

export function stripHexPrefix(value) {
  return value.startsWith("0x") ? value.slice(2) : value;
}

export function hexToBuffer(value) {
  return Buffer.from(stripHexPrefix(value), "hex");
}

export function toHex(value) {
  if (typeof value === "string") {
    return value.startsWith("0x") ? value : `0x${value}`;
  }
  return `0x${Buffer.from(value).toString("hex")}`;
}

export function ensure32ByteHex(value) {
  const hex = stripHexPrefix(value).padStart(64, "0").slice(0, 64);
  return `0x${hex}`;
}

export function computeRecordCommitment(record) {
  return toHex(sha256Hex({ domain: "uzima.record.v1", payload: normalize(record) }));
}

export function computeCredentialCommitment(credential) {
  return toHex(
    sha256Hex({ domain: "uzima.credential.v1", payload: normalize(credential) }),
  );
}

export function computeAddressCommitment(address) {
  return toHex(sha256Hex({ domain: "uzima.address.v1", address }));
}

export function computePseudonym(requester, issuer, recordId) {
  return toHex(
    sha256Hex({
      domain: "UZIMA_ZK_PSEUDONYM_V1",
      requester,
      issuer,
      record_id: Number(recordId),
    }),
  );
}

export function computePublicInputsHash(publicInputs) {
  return toHex(
    sha256Hex({
      domain: "uzima.zk.public_inputs.v1",
      payload: normalize(publicInputs),
    }),
  );
}

export function computeProofHash(proofHex) {
  return toHex(sha256Hex(hexToBuffer(proofHex)));
}

