import { groth16 } from "snarkjs";
import fs from "fs";

async function main() {
  const input = {
    recordHash: "123",
    userHash: "456",
    authorityHash: "789",
  };

  const { proof, publicSignals } = await groth16.fullProve(
    input,
    "medical_access.wasm",
    "medical_access.zkey"
  );

  fs.writeFileSync("proof.json", JSON.stringify({ proof, publicSignals }));
}

main();
