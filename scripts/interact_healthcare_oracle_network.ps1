param(
  [Parameter(Mandatory=$true)][string]$ContractId,
  [Parameter(Mandatory=$true)][string]$Network,
  [Parameter(Mandatory=$true)][string]$SourceAccount,
  [Parameter(Mandatory=$true)][string]$Method,
  [Parameter(ValueFromRemainingArguments=$true)][string[]]$ArgsList
)

$kv = @{}
foreach ($arg in $ArgsList) {
  if ($arg -notmatch "=") { throw "Invalid argument '$arg'. Use key=value format." }
  $parts = $arg -split '=',2
  $kv[$parts[0]] = $parts[1]
}

function Invoke-OracleMethod {
  param([string[]]$MethodArgs)
  soroban contract invoke --id $ContractId --network $Network --source-account $SourceAccount -- @MethodArgs
}

switch ($Method) {
  "register_oracle" {
    Invoke-OracleMethod @("register_oracle","--operator",$kv["operator"],"--endpoint",$kv["endpoint"],"--source_type",$kv["source_type"])
  }
  "verify_oracle" {
    Invoke-OracleMethod @("verify_oracle","--admin",$kv["admin"],"--operator",$kv["operator"],"--verified",$kv["verified"],"--active",$kv["active"])
  }
  "submit_drug_price" {
    Invoke-OracleMethod @("submit_drug_price","--operator",$kv["operator"],"--feed_id",$kv["feed_id"],"--ndc_code",$kv["ndc_code"],"--currency",$kv["currency"],"--price_minor",$kv["price_minor"],"--availability_units",$kv["availability_units"],"--observed_at",$kv["observed_at"])
  }
  "submit_treatment_outcome" {
    Invoke-OracleMethod @("submit_treatment_outcome","--operator",$kv["operator"],"--outcome_id",$kv["outcome_id"],"--condition_code",$kv["condition_code"],"--treatment_code",$kv["treatment_code"],"--improvement_rate_bps",$kv["improvement_rate_bps"],"--readmission_rate_bps",$kv["readmission_rate_bps"],"--mortality_rate_bps",$kv["mortality_rate_bps"],"--sample_size",$kv["sample_size"],"--reported_at",$kv["reported_at"])
  }
  "submit_clinical_trial" {
    Invoke-OracleMethod @("submit_clinical_trial","--operator",$kv["operator"],"--trial_id",$kv["trial_id"],"--phase",$kv["phase"],"--enrolled",$kv["enrolled"],"--success_rate_bps",$kv["success_rate_bps"],"--adverse_event_rate_bps",$kv["adverse_event_rate_bps"],"--result_hash",$kv["result_hash"],"--published_at",$kv["published_at"])
  }
  "submit_regulatory_update" {
    Invoke-OracleMethod @("submit_regulatory_update","--operator",$kv["operator"],"--regulation_id",$kv["regulation_id"],"--authority",$kv["authority"],"--status",$kv["status"],"--title",$kv["title"],"--details_hash",$kv["details_hash"],"--effective_at",$kv["effective_at"])
  }
  "raise_dispute" {
    Invoke-OracleMethod @("raise_dispute","--challenger",$kv["challenger"],"--kind",$kv["kind"],"--feed_id",$kv["feed_id"],"--reason",$kv["reason"])
  }
  "resolve_dispute" {
    $penalized = if ($kv.ContainsKey("penalized_oracle")) { $kv["penalized_oracle"] } else { "null" }
    Invoke-OracleMethod @("resolve_dispute","--resolver",$kv["resolver"],"--dispute_id",$kv["dispute_id"],"--valid_dispute",$kv["valid_dispute"],"--ruling",$kv["ruling"],"--penalized_oracle",$penalized)
  }
  "get_consensus" {
    Invoke-OracleMethod @("get_consensus","--kind",$kv["kind"],"--feed_id",$kv["feed_id"])
  }
  "get_oracle" {
    Invoke-OracleMethod @("get_oracle","--operator",$kv["operator"])
  }
  "get_dispute" {
    Invoke-OracleMethod @("get_dispute","--dispute_id",$kv["dispute_id"])
  }
  "get_config" {
    Invoke-OracleMethod @("get_config")
  }
  default {
    throw "Unsupported method: $Method"
  }
}
