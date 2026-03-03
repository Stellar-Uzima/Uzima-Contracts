#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 3 ]]; then
  echo "Usage: $0 <contract_id> <network> <source_account> <method> [key=value ...]"
  echo "Example: $0 CABC... testnet alice register_oracle operator=G... endpoint=https://o1 source_type=4"
  exit 1
fi

CONTRACT_ID="$1"
NETWORK="$2"
SOURCE_ACCOUNT="$3"
METHOD="${4:-}"
shift 4 || true

if [[ -z "$METHOD" ]]; then
  echo "Missing method"
  exit 1
fi

declare -A ARGS
for pair in "$@"; do
  if [[ "$pair" != *=* ]]; then
    echo "Invalid argument '$pair'. Use key=value format."
    exit 1
  fi
  key="${pair%%=*}"
  val="${pair#*=}"
  ARGS["$key"]="$val"
done

invoke() {
  soroban contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --source-account "$SOURCE_ACCOUNT" \
    -- "$@"
}

case "$METHOD" in
  register_oracle)
    invoke register_oracle \
      --operator "${ARGS[operator]}" \
      --endpoint "${ARGS[endpoint]}" \
      --source_type "${ARGS[source_type]}"
    ;;

  verify_oracle)
    invoke verify_oracle \
      --admin "${ARGS[admin]}" \
      --operator "${ARGS[operator]}" \
      --verified "${ARGS[verified]}" \
      --active "${ARGS[active]}"
    ;;

  submit_drug_price)
    invoke submit_drug_price \
      --operator "${ARGS[operator]}" \
      --feed_id "${ARGS[feed_id]}" \
      --ndc_code "${ARGS[ndc_code]}" \
      --currency "${ARGS[currency]}" \
      --price_minor "${ARGS[price_minor]}" \
      --availability_units "${ARGS[availability_units]}" \
      --observed_at "${ARGS[observed_at]}"
    ;;

  submit_treatment_outcome)
    invoke submit_treatment_outcome \
      --operator "${ARGS[operator]}" \
      --outcome_id "${ARGS[outcome_id]}" \
      --condition_code "${ARGS[condition_code]}" \
      --treatment_code "${ARGS[treatment_code]}" \
      --improvement_rate_bps "${ARGS[improvement_rate_bps]}" \
      --readmission_rate_bps "${ARGS[readmission_rate_bps]}" \
      --mortality_rate_bps "${ARGS[mortality_rate_bps]}" \
      --sample_size "${ARGS[sample_size]}" \
      --reported_at "${ARGS[reported_at]}"
    ;;

  submit_clinical_trial)
    invoke submit_clinical_trial \
      --operator "${ARGS[operator]}" \
      --trial_id "${ARGS[trial_id]}" \
      --phase "${ARGS[phase]}" \
      --enrolled "${ARGS[enrolled]}" \
      --success_rate_bps "${ARGS[success_rate_bps]}" \
      --adverse_event_rate_bps "${ARGS[adverse_event_rate_bps]}" \
      --result_hash "${ARGS[result_hash]}" \
      --published_at "${ARGS[published_at]}"
    ;;

  submit_regulatory_update)
    invoke submit_regulatory_update \
      --operator "${ARGS[operator]}" \
      --regulation_id "${ARGS[regulation_id]}" \
      --authority "${ARGS[authority]}" \
      --status "${ARGS[status]}" \
      --title "${ARGS[title]}" \
      --details_hash "${ARGS[details_hash]}" \
      --effective_at "${ARGS[effective_at]}"
    ;;

  finalize_feed)
    invoke finalize_feed \
      --kind "${ARGS[kind]}" \
      --feed_id "${ARGS[feed_id]}"
    ;;

  raise_dispute)
    invoke raise_dispute \
      --challenger "${ARGS[challenger]}" \
      --kind "${ARGS[kind]}" \
      --feed_id "${ARGS[feed_id]}" \
      --reason "${ARGS[reason]}"
    ;;

  resolve_dispute)
    if [[ -n "${ARGS[penalized_oracle]:-}" ]]; then
      invoke resolve_dispute \
        --resolver "${ARGS[resolver]}" \
        --dispute_id "${ARGS[dispute_id]}" \
        --valid_dispute "${ARGS[valid_dispute]}" \
        --ruling "${ARGS[ruling]}" \
        --penalized_oracle "${ARGS[penalized_oracle]}"
    else
      invoke resolve_dispute \
        --resolver "${ARGS[resolver]}" \
        --dispute_id "${ARGS[dispute_id]}" \
        --valid_dispute "${ARGS[valid_dispute]}" \
        --ruling "${ARGS[ruling]}" \
        --penalized_oracle null
    fi
    ;;

  get_consensus)
    invoke get_consensus \
      --kind "${ARGS[kind]}" \
      --feed_id "${ARGS[feed_id]}"
    ;;

  get_oracle)
    invoke get_oracle --operator "${ARGS[operator]}"
    ;;

  get_dispute)
    invoke get_dispute --dispute_id "${ARGS[dispute_id]}"
    ;;

  get_config)
    invoke get_config
    ;;

  *)
    echo "Unsupported method: $METHOD"
    echo "Supported: register_oracle, verify_oracle, submit_drug_price, submit_treatment_outcome, submit_clinical_trial, submit_regulatory_update, finalize_feed, raise_dispute, resolve_dispute, get_consensus, get_oracle, get_dispute, get_config"
    exit 1
    ;;
esac
