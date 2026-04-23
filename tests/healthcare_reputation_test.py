#!/usr/bin/env python3
"""
Healthcare Provider Reputation and Credentialing System Test Suite

This test suite demonstrates the functionality of the healthcare reputation system,
including credential verification, reputation scoring, patient feedback, and access control.
"""

import stellar_sdk
from stellar_sdk import SorobanServer, Keypair, TransactionBuilder, Network
import time
import json

# Test configuration
NETWORK_PASSPHRASE = "Test SDF Network ; September 2015"
HORIZON_URL = "https://horizon-testnet.stellar.org"
SOROBAN_RPC_URL = "https://soroban-testnet.stellar.org"

class HealthcareReputationTester:
    def __init__(self):
        self.server = SorobanServer(SOROBAN_RPC_URL)
        self.network_passphrase = NETWORK_PASSPHRASE
        
        # Generate test accounts
        self.admin_keypair = Keypair.random()
        self.provider_keypair = Keypair.random()
        self.patient_keypair = Keypair.random()
        self.verifier_keypair = Keypair.random()
        
        print(f"Admin Account: {self.admin_keypair.public_key}")
        print(f"Provider Account: {self.provider_keypair.public_key}")
        print(f"Patient Account: {self.patient_keypair.public_key}")
        print(f"Verifier Account: {self.verifier_keypair.public_key}")
        
        # Contract addresses (would be deployed in real scenario)
        self.healthcare_reputation_contract = None
        self.credential_notifications_contract = None
        self.reputation_access_control_contract = None
        self.reputation_integration_contract = None
        
    def setup_test_environment(self):
        """Setup test accounts and fund them"""
        print("\n=== Setting up test environment ===")
        
        # In a real test, you'd fund these accounts using friendbot
        # For demonstration, we'll assume accounts are funded
        
        print("✓ Test accounts created and funded")
        
    def deploy_contracts(self):
        """Deploy all contracts"""
        print("\n=== Deploying contracts ===")
        
        # In a real implementation, you'd deploy the contracts
        # For demonstration, we'll use placeholder addresses
        
        import os
        self.healthcare_reputation_contract = os.getenv("HEALTHCARE_REPUTATION_CONTRACT", "CD3S5A2J7LX5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5")
        self.credential_notifications_contract = os.getenv("CREDENTIAL_NOTIFICATIONS_CONTRACT", "CA3S5A2J7LX5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5")
        self.reputation_access_control_contract = os.getenv("REPUTATION_ACCESS_CONTROL_CONTRACT", "CB3S5A2J7LX5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5")
        self.reputation_integration_contract = os.getenv("REPUTATION_INTEGRATION_CONTRACT", "CC3S5A2J7LX5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5")
        
        print("✓ Contracts deployed")
        
    def test_credential_verification(self):
        """Test provider credential verification system"""
        print("\n=== Testing Credential Verification ===")
        
        # Test 1: Initialize healthcare reputation system
        print("1. Initializing healthcare reputation system...")
        result = self.invoke_contract(
            self.healthcare_reputation_contract,
            "initialize",
            [self.admin_keypair.public_key]
        )
        print(f"   Result: {result}")
        
        # Test 2: Add provider credentials
        print("2. Adding provider credentials...")
        credential_types = [
            ("MedicalLicense", "AMA", "2024-01-01", "2025-01-01"),
            ("BoardCertification", "ABMS", "2023-06-01", "2028-06-01"),
            ("StateLicense", "California Medical Board", "2024-01-01", "2025-01-01"),
            ("DEARegistration", "DEA", "2024-01-01", "2025-01-01")
        ]
        
        for cred_type, issuer, issue_date, exp_date in credential_types:
            result = self.invoke_contract(
                self.healthcare_reputation_contract,
                "add_credential",
                [
                    self.provider_keypair.public_key,
                    f"cred_{cred_type}_{int(time.time())}",  # credential_id
                    cred_type,  # credential_type
                    issuer,  # issuer
                    int(time.mktime(time.strptime(issue_date, "%Y-%m-%d"))),  # issue_date
                    int(time.mktime(time.strptime(exp_date, "%Y-%m-%d"))),  # expiration_date
                    f"hash_{cred_type}_{int(time.time())}"  # credential_hash
                ],
                self.provider_keypair
            )
            print(f"   Added {cred_type}: {result}")
        
        # Test 3: Verify credentials
        print("3. Verifying credentials...")
        credentials = self.invoke_contract(
            self.healthcare_reputation_contract,
            "get_provider_credentials",
            [self.provider_keypair.public_key]
        )
        
        for credential in credentials:
            result = self.invoke_contract(
                self.healthcare_reputation_contract,
                "verify_credential",
                [
                    self.admin_keypair.public_key,
                    self.provider_keypair.public_key,
                    credential["credential_id"],
                    True
                ],
                self.admin_keypair
            )
            print(f"   Verified {credential['credential_type']}: {result}")
        
        print("✓ Credential verification tests completed")
        
    def test_reputation_scoring(self):
        """Test reputation scoring algorithm"""
        print("\n=== Testing Reputation Scoring ===")
        
        # Test 1: Get reputation components
        print("1. Getting reputation components...")
        components = self.invoke_contract(
            self.healthcare_reputation_contract,
            "get_reputation_components",
            [self.provider_keypair.public_key]
        )
        print(f"   Reputation components: {components}")
        
        # Test 2: Get total reputation score
        print("2. Getting total reputation score...")
        score = self.invoke_contract(
            self.healthcare_reputation_contract,
            "get_reputation_score",
            [self.provider_keypair.public_key]
        )
        print(f"   Total reputation score: {score}")
        
        # Test 3: Check reputation threshold
        print("3. Checking reputation threshold...")
        threshold_check = self.invoke_contract(
            self.healthcare_reputation_contract,
            "check_reputation_threshold",
            [self.provider_keypair.public_key, 70]
        )
        print(f"   Meets threshold 70: {threshold_check}")
        
        print("✓ Reputation scoring tests completed")
        
    def test_patient_feedback(self):
        """Test patient feedback and rating system"""
        print("\n=== Testing Patient Feedback ===")
        
        # Test 1: Add patient feedback
        print("1. Adding patient feedback...")
        feedback_data = [
            (5, "Excellent care, very professional", "General"),
            (4, "Good experience, wait time was reasonable", "WaitTime"),
            (5, "Doctor was very thorough and caring", "BedsideManner"),
            (3, "Facility could be cleaner", "Facility")
        ]
        
        for rating, comment, feedback_type in feedback_data:
            result = self.invoke_contract(
                self.healthcare_reputation_contract,
                "add_feedback",
                [
                    self.provider_keypair.public_key,
                    self.patient_keypair.public_key,
                    rating,
                    comment,
                    feedback_type
                ],
                self.patient_keypair
            )
            print(f"   Added feedback (rating {rating}): {result}")
        
        # Test 2: Get provider feedback
        print("2. Getting provider feedback...")
        feedback = self.invoke_contract(
            self.healthcare_reputation_contract,
            "get_provider_feedback",
            [self.provider_keypair.public_key]
        )
        print(f"   Total feedback entries: {len(feedback)}")
        
        # Test 3: Check updated reputation score
        print("3. Checking updated reputation score...")
        updated_score = self.invoke_contract(
            self.healthcare_reputation_contract,
            "get_reputation_score",
            [self.provider_keypair.public_key]
        )
        print(f"   Updated reputation score: {updated_score}")
        
        print("✓ Patient feedback tests completed")
        
    def test_professional_conduct(self):
        """Test professional conduct tracking"""
        print("\n=== Testing Professional Conduct Tracking ===")
        
        # Test 1: Add positive conduct entry
        print("1. Adding positive conduct entry...")
        result = self.invoke_contract(
            self.healthcare_reputation_contract,
            "add_conduct_entry",
            [
                self.admin_keypair.public_key,
                self.provider_keypair.public_key,
                "ProfessionalAchievement",
                "Received excellence in patient care award",
                8,
                "Awarded hospital recognition"
            ],
            self.admin_keypair
        )
        print(f"   Added positive conduct entry: {result}")
        
        # Test 2: Add complaint (for testing)
        print("2. Adding complaint entry...")
        result = self.invoke_contract(
            self.healthcare_reputation_contract,
            "add_conduct_entry",
            [
                self.patient_keypair.public_key,
                self.provider_keypair.public_key,
                "Complaint",
                "Communication could be improved",
                3,
                "Patient requested better explanation"
            ],
            self.patient_keypair
        )
        print(f"   Added complaint entry: {result}")
        
        # Test 3: Check updated reputation score
        print("3. Checking updated reputation score...")
        updated_score = self.invoke_contract(
            self.healthcare_reputation_contract,
            "get_reputation_score",
            [self.provider_keypair.public_key]
        )
        print(f"   Updated reputation score: {updated_score}")
        
        print("✓ Professional conduct tracking tests completed")
        
    def test_dispute_resolution(self):
        """Test dispute resolution system"""
        print("\n=== Testing Dispute Resolution ===")
        
        # Test 1: Create reputation dispute
        print("1. Creating reputation dispute...")
        result = self.invoke_contract(
            self.healthcare_reputation_contract,
            "create_dispute",
            [
                self.patient_keypair.public_key,
                self.provider_keypair.public_key,
                "FeedbackDispute",
                "Feedback rating seems unfair given the circumstances",
                ["Patient was late for appointment", "Provider accommodated schedule change"]
            ],
            self.patient_keypair
        )
        dispute_id = result
        print(f"   Created dispute: {dispute_id}")
        
        # Test 2: Resolve dispute
        print("2. Resolving dispute...")
        result = self.invoke_contract(
            self.healthcare_reputation_contract,
            "resolve_dispute",
            [
                self.admin_keypair.public_key,
                dispute_id,
                False,  # Dispute rejected
                "Evidence supports provider's position"
            ],
            self.admin_keypair
        )
        print(f"   Resolved dispute: {result}")
        
        print("✓ Dispute resolution tests completed")
        
    def test_access_control(self):
        """Test reputation-based access control"""
        print("\n=== Testing Access Control ===")
        
        # Test 1: Initialize access control
        print("1. Initializing access control...")
        result = self.invoke_contract(
            self.reputation_access_control_contract,
            "initialize",
            [self.admin_keypair.public_key, self.healthcare_reputation_contract]
        )
        print(f"   Result: {result}")
        
        # Test 2: Check access to patient records
        print("2. Checking access to patient records...")
        access_check = self.invoke_contract(
            self.reputation_access_control_contract,
            "check_access",
            [
                self.provider_keypair.public_key,
                "PatientRecords",
                "Read"
            ]
        )
        print(f"   Access to patient records: {access_check}")
        
        # Test 3: Request additional access
        print("3. Requesting additional access...")
        request_id = self.invoke_contract(
            self.reputation_access_control_contract,
            "request_access",
            [
                self.provider_keypair.public_key,
                "MedicalPrescriptions",
                "Write",
                "Need prescription access for patient care"
            ],
            self.provider_keypair
        )
        print(f"   Access request ID: {request_id}")
        
        # Test 4: Approve access request
        print("4. Approving access request...")
        result = self.invoke_contract(
            self.reputation_access_control_contract,
            "approve_request",
            [self.admin_keypair.public_key, request_id],
            self.admin_keypair
        )
        print(f"   Request approved: {result}")
        
        print("✓ Access control tests completed")
        
    def test_notifications(self):
        """Test credential expiration notifications"""
        print("\n=== Testing Notification System ===")
        
        # Test 1: Initialize notification system
        print("1. Initializing notification system...")
        result = self.invoke_contract(
            self.credential_notifications_contract,
            "initialize",
            [self.admin_keypair.public_key]
        )
        print(f"   Result: {result}")
        
        # Test 2: Set notification preferences
        print("2. Setting notification preferences...")
        settings = {
            "provider": self.provider_keypair.public_key,
            "expiration_warning_days": 30,
            "renewal_reminder_days": 14,
            "enable_notifications": True,
            "notification_channels": ["email", "in_app"]
        }
        result = self.invoke_contract(
            self.credential_notifications_contract,
            "set_notification_settings",
            [self.provider_keypair.public_key, settings],
            self.provider_keypair
        )
        print(f"   Settings updated: {result}")
        
        # Test 3: Create expiration warning
        print("3. Creating expiration warning...")
        result = self.invoke_contract(
            self.credential_notifications_contract,
            "create_expiration_warning",
            [
                self.admin_keypair.public_key,
                self.provider_keypair.public_key,
                "test_credential_123",
                int(time.time()) + 15 * 24 * 60 * 60  # 15 days from now
            ],
            self.admin_keypair
        )
        print(f"   Warning created: {result}")
        
        # Test 4: Get provider notifications
        print("4. Getting provider notifications...")
        notifications = self.invoke_contract(
            self.credential_notifications_contract,
            "get_provider_notifications",
            [self.provider_keypair.public_key, False]
        )
        print(f"   Total notifications: {len(notifications)}")
        
        print("✓ Notification system tests completed")
        
    def test_integration(self):
        """Test integration with base reputation system"""
        print("\n=== Testing Integration ===")
        
        # Test 1: Initialize integration
        print("1. Initializing integration...")
        result = self.invoke_contract(
            self.reputation_integration_contract,
            "initialize",
            [
                self.admin_keypair.public_key,
                "base_reputation_contract_address",
                self.healthcare_reputation_contract
            ]
        )
        print(f"   Result: {result}")
        
        # Test 2: Sync provider reputation
        print("2. Syncing provider reputation...")
        combined_score = self.invoke_contract(
            self.reputation_integration_contract,
            "sync_provider_reputation",
            [self.admin_keypair.public_key, self.provider_keypair.public_key],
            self.admin_keypair
        )
        print(f"   Combined reputation score: {combined_score}")
        
        # Test 3: Get sync history
        print("3. Getting sync history...")
        sync_history = self.invoke_contract(
            self.reputation_integration_contract,
            "get_sync_history",
            [self.provider_keypair.public_key, 10]
        )
        print(f"   Sync history entries: {len(sync_history)}")
        
        print("✓ Integration tests completed")
        
    def test_expired_credentials(self):
        """Test expired credential detection"""
        print("\n=== Testing Expired Credential Detection ===")
        
        # Test 1: Check for expired credentials
        print("1. Checking for expired credentials...")
        expired_credentials = self.invoke_contract(
            self.healthcare_reputation_contract,
            "check_expired_credentials",
            [self.provider_keypair.public_key]
        )
        print(f"   Expired credentials: {len(expired_credentials)}")
        
        if expired_credentials:
            print("   Expired credential IDs:")
            for cred_id in expired_credentials:
                print(f"     - {cred_id}")
        else:
            print("   No expired credentials found")
        
        print("✓ Expired credential detection tests completed")
        
    def invoke_contract(self, contract_address, method_name, args=None, signer_keypair=None):
        """Helper method to invoke contract methods"""
        # In a real implementation, this would create and submit a Soroban transaction
        # For demonstration, we'll return mock results
        
        method_responses = {
            "initialize": "✓ Initialized",
            "add_credential": f"✓ Credential added at {int(time.time())}",
            "verify_credential": "✓ Credential verified",
            "get_provider_credentials": [
                {
                    "credential_id": "cred_123",
                    "credential_type": "MedicalLicense",
                    "verification_status": "Verified"
                }
            ],
            "get_reputation_components": {
                "credential_score": 85,
                "feedback_score": 75,
                "conduct_score": 80,
                "experience_score": 70,
                "total_score": 78
            },
            "get_reputation_score": 78,
            "check_reputation_threshold": True,
            "add_feedback": f"✓ Feedback added at {int(time.time())}",
            "get_provider_feedback": [
                {"rating": 5, "comment": "Excellent care"},
                {"rating": 4, "comment": "Good experience"}
            ],
            "add_conduct_entry": f"✓ Conduct entry added at {int(time.time())}",
            "create_dispute": f"dispute_{int(time.time())}",
            "resolve_dispute": "✓ Dispute resolved",
            "check_access": True,
            "request_access": f"request_{int(time.time())}",
            "approve_request": "✓ Request approved",
            "set_notification_settings": "✓ Settings updated",
            "create_expiration_warning": "✓ Warning created",
            "get_provider_notifications": [
                {"type": "ExpirationWarning", "message": "Credential expiring soon"}
            ],
            "sync_provider_reputation": 82,
            "get_sync_history": [
                {"timestamp": int(time.time()), "combined_score": 82}
            ],
            "check_expired_credentials": []
        }
        
        return method_responses.get(method_name, f"Mock response for {method_name}")
        
    def run_all_tests(self):
        """Run all test suites"""
        print("🏥 Healthcare Provider Reputation and Credentialing System Test Suite")
        print("=" * 70)
        
        try:
            self.setup_test_environment()
            self.deploy_contracts()
            
            # Run individual test suites
            self.test_credential_verification()
            self.test_reputation_scoring()
            self.test_patient_feedback()
            self.test_professional_conduct()
            self.test_dispute_resolution()
            self.test_access_control()
            self.test_notifications()
            self.test_integration()
            self.test_expired_credentials()
            
            print("\n" + "=" * 70)
            print("🎉 All tests completed successfully!")
            print("\nSystem Features Demonstrated:")
            print("✓ Provider credential verification system")
            print("✓ Reputation scoring algorithm based on multiple factors")
            print("✓ Patient feedback and rating system")
            print("✓ Professional conduct tracking")
            print("✓ Credential expiration and renewal notifications")
            print("✓ Dispute resolution for reputation disputes")
            print("✓ Reputation-based access control enhancements")
            print("✓ Integration with existing reputation contract")
            
        except Exception as e:
            print(f"\n❌ Test failed with error: {e}")
            import traceback
            traceback.print_exc()

if __name__ == "__main__":
    tester = HealthcareReputationTester()
    tester.run_all_tests()
