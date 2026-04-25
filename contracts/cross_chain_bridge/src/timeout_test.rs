#[cfg(test)]
mod tests {
    use crate::{CrossChainBridgeContract, OperationType, OperationStatus, Error, VERIFICATION_TIMEOUT, MESSAGE_PASSING_TIMEOUT, TOKEN_TRANSFER_TIMEOUT, MAX_EXTENSIONS};
    use soroban_sdk::{BytesN, Env, Address};
    use soroban_sdk::testutils::{Ledger, Address as AddressTestUtils};

    #[test]
    fn test_timeout_creation_and_check() {
        let env = Env::default();
        let admin = < Address as AddressTestUtils>::generate(&env);
        let refund_address = < Address as AddressTestUtils>::generate(&env);
        
        // Initialize contract
        CrossChainBridgeContract::initialize(
            env.clone(),
            admin.clone(),
            < Address as AddressTestUtils>::generate(&env),
            < Address as AddressTestUtils>::generate(&env),
            < Address as AddressTestUtils>::generate(&env),
        ).unwrap();

        // Create operation
        let op_id = BytesN::from_array(&env, &[1; 32]);
        CrossChainBridgeContract::create_operation(
            env.clone(),
            admin.clone(),
            op_id.clone(),
            OperationType::TokenTransfer,
            refund_address.clone(),
        ).unwrap();

        // Get operation and verify deadline
        let operation = CrossChainBridgeContract::get_operation(env.clone(), op_id.clone()).unwrap();
        assert_eq!(operation.status, OperationStatus::Pending);
        assert_eq!(operation.refund_address, refund_address);
        assert_eq!(operation.op_type, OperationType::TokenTransfer);
        assert_eq!(operation.extended_count, 0);

        // Mock time passing beyond deadline
        env.ledger().set_timestamp(operation.deadline + 100);

        // Check timeout should trigger refund
        CrossChainBridgeContract::check_timeout(env.clone(), op_id.clone()).unwrap();

        // Verify operation is now refunded
        let refunded_op = CrossChainBridgeContract::get_operation(env.clone(), op_id.clone()).unwrap();
        assert_eq!(refunded_op.status, OperationStatus::Refunded);
    }

    #[test]
    fn test_timeout_extension() {
        let env = Env::default();
        let admin = < Address as AddressTestUtils>::generate(&env);
        let refund_address = < Address as AddressTestUtils>::generate(&env);
        
        // Initialize contract
        CrossChainBridgeContract::initialize(
            env.clone(),
            admin.clone(),
            < Address as AddressTestUtils>::generate(&env),
            < Address as AddressTestUtils>::generate(&env),
            < Address as AddressTestUtils>::generate(&env),
        ).unwrap();

        // Create operation
        let op_id = BytesN::from_array(&env, &[2; 32]);
        CrossChainBridgeContract::create_operation(
            env.clone(),
            admin.clone(),
            op_id.clone(),
            OperationType::MessagePassing,
            refund_address.clone(),
        ).unwrap();

        let original_deadline = CrossChainBridgeContract::get_operation(env.clone(), op_id.clone()).unwrap().deadline;

        // Extend timeout
        CrossChainBridgeContract::extend_timeout(
            env.clone(),
            admin.clone(),
            op_id.clone(),
            3600, // 1 hour extension
        ).unwrap();

        let extended_op = CrossChainBridgeContract::get_operation(env.clone(), op_id.clone()).unwrap();
        assert_eq!(extended_op.status, OperationStatus::Extended);
        assert_eq!(extended_op.extended_count, 1);
        assert!(extended_op.deadline > original_deadline);
    }

    #[test]
    fn test_default_timeouts() {
        let env = Env::default();
        
        // Test different operation types have correct default timeouts
        assert_eq!(
            CrossChainBridgeContract::get_default_timeout_internal(env.clone(), OperationType::TokenTransfer),
            TOKEN_TRANSFER_TIMEOUT
        );
        assert_eq!(
            CrossChainBridgeContract::get_default_timeout_internal(env.clone(), OperationType::MessagePassing),
            MESSAGE_PASSING_TIMEOUT
        );
        assert_eq!(
            CrossChainBridgeContract::get_default_timeout_internal(env.clone(), OperationType::Verification),
            VERIFICATION_TIMEOUT
        );
    }

    #[test]
    fn test_max_extensions_limit() {
        let env = Env::default();
        let admin = < Address as AddressTestUtils>::generate(&env);
        let refund_address = < Address as AddressTestUtils>::generate(&env);
        
        // Initialize contract
        CrossChainBridgeContract::initialize(
            env.clone(),
            admin.clone(),
            < Address as AddressTestUtils>::generate(&env),
            < Address as AddressTestUtils>::generate(&env),
            < Address as AddressTestUtils>::generate(&env),
        ).unwrap();

        // Create operation
        let op_id = BytesN::from_array(&env, &[3; 32]);
        CrossChainBridgeContract::create_operation(
            env.clone(),
            admin.clone(),
            op_id.clone(),
            OperationType::Verification,
            refund_address.clone(),
        ).unwrap();

        // Extend timeout maximum number of times
        for _ in 0..MAX_EXTENSIONS {
            CrossChainBridgeContract::extend_timeout(
                env.clone(),
                admin.clone(),
                op_id.clone(),
                100,
            ).unwrap();
        }

        // Try to extend one more time - should fail
        let result = CrossChainBridgeContract::extend_timeout(
            env.clone(),
            admin.clone(),
            op_id.clone(),
            100,
        );
        assert_eq!(result.unwrap_err(), Error::MaxExtensionsReached);
    }
}
