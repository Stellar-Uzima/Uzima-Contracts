#[cfg(test)]
mod tests {
    use crate::{
        CrossChainBridgeContract, OperationStatus, OperationType, MESSAGE_PASSING_TIMEOUT,
        TOKEN_TRANSFER_TIMEOUT, VERIFICATION_TIMEOUT,
    };
    use soroban_sdk::Env;

    #[test]
    fn test_default_timeouts() {
        let env = Env::default();

        // Test different operation types have correct default timeouts
        assert_eq!(
            CrossChainBridgeContract::get_default_timeout_internal(
                env.clone(),
                OperationType::TokenTransfer
            ),
            TOKEN_TRANSFER_TIMEOUT
        );
        assert_eq!(
            CrossChainBridgeContract::get_default_timeout_internal(
                env.clone(),
                OperationType::MessagePassing
            ),
            MESSAGE_PASSING_TIMEOUT
        );
        assert_eq!(
            CrossChainBridgeContract::get_default_timeout_internal(
                env.clone(),
                OperationType::Verification
            ),
            VERIFICATION_TIMEOUT
        );
    }

    #[test]
    fn test_operation_type_copy() {
        // Test that OperationType implements Copy trait
        let op_type = OperationType::TokenTransfer;
        let copied = op_type;
        assert_eq!(op_type, copied);
    }

    #[test]
    fn test_operation_status_copy() {
        // Test that OperationStatus implements Copy trait
        let status = OperationStatus::Pending;
        let copied = status;
        assert_eq!(status, copied);
    }
}
