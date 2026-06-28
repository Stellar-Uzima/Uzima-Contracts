#!/bin/bash

# Color definitions
BLUE='\033[1;34m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
RED='\033[1;31m'
NC='\033[0m' # No Color

# Function to print a header
print_header() {
    echo -e "${BLUE}=======================================================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}=======================================================================${NC}"
}

# Function to pause and wait for user input
wait_for_user() {
    echo -e "${YELLOW}"
    read -p "  Press Enter to continue..."
    echo -e "${NC}"
}

# --- Script Start ---

print_header "Patient Record Lifecycle Demo"
echo -e "This script demonstrates the full lifecycle of a patient record, from registration to access control."
wait_for_user

# 1. Deploy Contracts
print_header "Step 1: Deploying Contracts"
echo "First, we deploy the necessary smart contracts to a local network."
echo "This includes the patient registry, medical records, and consent management contracts."
echo
echo -e "${GREEN}make deploy-local${NC}"
make deploy-local
echo
echo "Contracts have been deployed."
wait_for_user

# 2. Register Patient
print_header "Step 2: Registering a New Patient"
echo "Now, we'll register a new patient in the system."
echo "This involves creating a new patient identity and storing it in the patient registry."
echo
echo -e "${GREEN}./scripts/interact.sh register_patient 'John Doe' '{"age": 30, "blood_type": "O+"}'${NC}"
./scripts/interact.sh register_patient 'John Doe' '{"age": 30, "blood_type": "O+"}'
echo
echo "Patient 'John Doe' is now registered."
wait_for_user

# 3. Write Medical Record
print_header "Step 3: Writing a Medical Record"
echo "Next, we'll add a new medical record for our patient."
echo "This record is encrypted and stored securely on-chain."
echo
echo -e "${GREEN}./scripts/interact.sh write_record 'John Doe' '{"diagnosis": "Hypertension", "medication": "Lisinopril"}'${NC}"
./scripts/interact.sh write_record 'John Doe' '{"diagnosis": "Hypertension", "medication": "Lisinopril"}'
echo
echo "A new medical record has been added for 'John Doe'."
wait_for_user

# 4. Grant Consent
print_header "Step 4: Granting Consent"
echo "The patient now grants consent to a doctor to read their medical records."
echo "This is a critical step in maintaining patient privacy and control."
echo
echo -e "${GREEN}./scripts/interact.sh grant_consent 'John Doe' 'Dr. Smith'${NC}"
./scripts/interact.sh grant_consent 'John Doe' 'Dr. Smith'
echo
echo "Consent has been granted to 'Dr. Smith'."
wait_for_user

# 5. Read Medical Record
print_header "Step 5: Reading the Medical Record"
echo "'Dr. Smith' will now read the patient's medical record."
echo "This is only possible because the patient has granted consent."
echo
echo -e "${GREEN}./scripts/interact.sh read_record 'John Doe' 'Dr. Smith'${NC}"
./scripts/interact.sh read_record 'John Doe' 'Dr. Smith'
echo
echo "Dr. Smith has successfully accessed the record."
wait_for_user

# 6. Revoke Consent
print_header "Step 6: Revoking Consent"
echo "The patient can revoke consent at any time."
echo "Let's revoke 'Dr. Smith's' access."
echo
echo -e "${GREEN}./scripts/interact.sh revoke_consent 'John Doe' 'Dr. Smith'${NC}"
./scripts/interact.sh revoke_consent 'John Doe' 'Dr. Smith'
echo
echo "Consent has been revoked for 'Dr. Smith'."
wait_for_user

# 7. Deny Access
print_header "Step 7: Denying Access"
echo "Now, if 'Dr. Smith' tries to read the record again, access will be denied."
echo "This demonstrates that the patient has full control over their data."
echo
echo -e "${RED}./scripts/interact.sh read_record 'John Doe' 'Dr. Smith'${NC}"
./scripts/interact.sh read_record 'John Doe' 'Dr. Smith'
echo
echo "Access was denied, as expected."
wait_for_user

print_header "Demo Complete"
echo "This demonstration has shown the complete, patient-controlled lifecycle of a medical record."
echo "Thank you for watching!"