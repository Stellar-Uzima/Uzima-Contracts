param(
  [Parameter(Mandatory=$true)][string]$ContractId,
  [Parameter(Mandatory=$true)][string]$Network,
  [Parameter(Mandatory=$true)][string]$SourceAccount,
  [Parameter(Mandatory=$true)][string]$AdminAddress,
  [Parameter(Mandatory=$true)][string]$ArbiterAddress,
  [Parameter(Mandatory=$true)][string]$OracleAddress,
  [Parameter(Mandatory=$true)][string]$ChallengerAddress,
  [Parameter(Mandatory=$true)][UInt64]$BaseTimestamp
)

$scriptPath = Join-Path $PSScriptRoot "interact_healthcare_oracle_network.ps1"

Write-Host "[1/7] Registering oracle source"
& $scriptPath $ContractId $Network $SourceAccount register_oracle "operator=$OracleAddress" "endpoint=https://oracle.demo.health" "source_type=4" | Out-Null

Write-Host "[2/7] Verifying oracle source"
& $scriptPath $ContractId $Network $SourceAccount verify_oracle "admin=$AdminAddress" "operator=$OracleAddress" "verified=true" "active=true" | Out-Null

Write-Host "[3/7] Submitting drug pricing update"
& $scriptPath $ContractId $Network $SourceAccount submit_drug_price "operator=$OracleAddress" "feed_id=NDC:55513-1234-1:KE" "ndc_code=55513-1234-1" "currency=USD" "price_minor=1050" "availability_units=220" "observed_at=$BaseTimestamp" | Out-Null

Write-Host "[4/7] Submitting treatment outcome update"
& $scriptPath $ContractId $Network $SourceAccount submit_treatment_outcome "operator=$OracleAddress" "outcome_id=OUTCOME:CHF:ACEI:2026Q1" "condition_code=I50.9" "treatment_code=ACEI" "improvement_rate_bps=7100" "readmission_rate_bps=950" "mortality_rate_bps=180" "sample_size=1200" "reported_at=$($BaseTimestamp+10)" | Out-Null

Write-Host "[5/7] Submitting clinical trial and regulatory updates"
& $scriptPath $ContractId $Network $SourceAccount submit_clinical_trial "operator=$OracleAddress" "trial_id=NCT-2026-001" "phase=3" "enrolled=450" "success_rate_bps=8200" "adverse_event_rate_bps=600" "result_hash=sha256:trial-a" "published_at=$($BaseTimestamp+20)" | Out-Null
& $scriptPath $ContractId $Network $SourceAccount submit_regulatory_update "operator=$OracleAddress" "regulation_id=FDA-2026-DRUG-UPDATE-11" "authority=1" "status=4" "title=Updated-Labeling-Requirement" "details_hash=sha256:reg-update-11" "effective_at=$($BaseTimestamp+30)" | Out-Null

Write-Host "[6/7] Raising dispute"
$disputeRaw = & $scriptPath $ContractId $Network $SourceAccount raise_dispute "challenger=$ChallengerAddress" "kind=3" "feed_id=FDA-2026-DRUG-UPDATE-11" "reason=Source-mismatch"
$disputeId = [regex]::Matches(($disputeRaw | Out-String), '\d+') | Select-Object -Last 1 | ForEach-Object { $_.Value }
if (-not $disputeId) { throw "Unable to parse dispute id from output: $disputeRaw" }

Write-Host "[7/7] Resolving dispute"
& $scriptPath $ContractId $Network $SourceAccount resolve_dispute "resolver=$ArbiterAddress" "dispute_id=$disputeId" "valid_dispute=true" "ruling=Confirmed-mismatch" "penalized_oracle=$OracleAddress" | Out-Null

Write-Host "Scenario completed. Dispute ID: $disputeId"
