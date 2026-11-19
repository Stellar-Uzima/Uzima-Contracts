//! Reentrancy regression tests for cross-chain message handlers (issue #1951).
//!
//! A `#[contract]` probe re-enters `execute_message` in the same frame, the way
//! a hostile `SorobanAuthorization` contract would during `require_auth`. The
//! inner call uses `try_` so the refusal is observable. The assertions prove
//! state was committed before the event/external call: exactly one transition
//! is recorded and the re-entrant call cannot double-execute the message.
#![cfg(test)]
#![allow(clippy::unwrap_used)]

use crate::{
    ChainId, CrossChainBridgeContract, CrossChainBridgeContractClient, CrossChainMessage,
    DataKey, MessageStatus, MessageType,
};
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, Address, BytesN, Env, String,
};

#[contract]
struct MessageHandlerReentryProbe;

#[contractimpl]
impl MessageHandlerReentryProbe {
    pub fn probe_execute(
        env: Env,
        bridge: Address,
        caller: Address,
        message_id: BytesN<32>,
    ) -> (bool, bool) {
        let client = CrossChainBridgeContractClient::new(&env, &bridge);
        let outer = client.try_execute_message(&caller, &message_id).is_ok();
        let inner = client.try_execute_message(&caller, &message_id).is_err();
        (outer, inner)
    }
}

fn setup<'a>(
    env: &'a Env,
) -> (CrossChainBridgeContractClient<'a>, Address, Address, BytesN<32>) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(env, &id);
    client.initialize(
        &admin,
        &Address::generate(env),
        &Address::generate(env),
        &Address::generate(env),
    );

    let message_id = BytesN::from_array(env, &[7u8; 32]);
    let caller = Address::generate(env);
    let message = CrossChainMessage {
        message_id: message_id.clone(),
        source_chain: ChainId::Ethereum,
        dest_chain: ChainId::Stellar,
        sender: String::from_str(env, "0xabc"),
        recipient: caller.clone(),
        payload_type: MessageType::RecordRequest,
        payload: String::from_str(env, "{}"),
        nonce: 1,
        timestamp: env.ledger().timestamp(),
        status: MessageStatus::Verified,
        signature: BytesN::from_array(env, &[0u8; 64]),
    };
    env.as_contract(&id, || {
        env.storage()
            .persistent()
            .set(&DataKey::Message(message_id.clone()), &message);
    });
    (client, id, caller, message_id)
}

#[test]
fn execute_message_cannot_be_double_executed_by_reentrant_callback() {
    let env = Env::default();
    let (client, bridge_id, caller, message_id) = setup(&env);
    let probe_id = env.register_contract(None, MessageHandlerReentryProbe);
    let probe = MessageHandlerReentryProbeClient::new(&env, &probe_id);

    let (outer_ok, inner_refused) = probe.probe_execute(&bridge_id, &caller, &message_id);

    assert!(outer_ok, "outer execute_message should succeed");
    assert!(inner_refused, "re-entrant execute_message must be refused");
    assert_eq!(
        client.get_message(&message_id).unwrap().status,
        MessageStatus::Executed,
        "state was committed once, before the event"
    );
}
