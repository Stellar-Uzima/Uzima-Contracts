# Measure storage and CPU cost reduction per call for Uzima contracts.
Write-Host "🧪 Running Soroban Contract Budget Measurement..." -ForegroundColor Cyan
Write-Host "-----------------------------------------------"

# Run tests and filter for CPU and storage budget measurements
cargo test --manifest-path contracts/medical_records/Cargo.toml -- --nocapture 2>&1 | Select-String "SNAPSHOT", "cpu", "budget", "storage", "instructions", "opt", "savings"
cargo test --package cross_chain_bridge -- --nocapture 2>&1 | Select-String "SNAPSHOT", "cpu", "budget", "storage", "instructions", "opt", "savings"
cargo test --package governor -- --nocapture 2>&1 | Select-String "SNAPSHOT", "cpu", "budget", "storage", "instructions", "opt", "savings"

Write-Host "-----------------------------------------------"
Write-Host "✅ Measurement complete." -ForegroundColor Green
