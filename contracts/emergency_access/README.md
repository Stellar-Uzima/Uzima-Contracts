# Emergency Access Contract

A comprehensive emergency access system for healthcare providers to access critical medical records in emergency situations while maintaining audit trails and temporary access controls.

## Features

- **Emergency Access Request Workflow**: Healthcare providers can request emergency access to patient records
- **Multi-Authority Approval System**: Configurable approval requirements with weighted authorities
- **Time-Limited Access Grants**: Automatic expiration of emergency access
- **Multi-Factor Authentication**: Support for various MFA factors (password, biometric, hardware token, etc.)
- **Comprehensive Audit Logging**: Full audit trail of all emergency access activities
- **Automatic Cleanup**: Background cleanup of expired grants
- **Dispute Resolution**: Integration with dispute resolution system for contested access
- **Compliance Reporting**: Detailed compliance reports for regulatory requirements

## Architecture

The contract integrates with existing healthcare infrastructure:

- **Medical Records Contract**: For record access validation
- **Identity Registry**: For DID-based identity verification
- **Governor Contract**: For approval workflows
- **Dispute Resolution**: For handling access disputes
- **Healthcare Compliance**: For audit logging and compliance reporting

## Key Components

### EmergencyRequest
Represents a request for emergency access with:
- Requester and patient information
- Emergency type and clinical justification
- Record scope (specific records or all)
- MFA factors provided
- Approval tracking

### EmergencyGrant
Active access grants with:
- Time-limited access
- Record scope limitations
- Access tracking and counting
- Automatic expiration

### EmergencyAuthority
Registered healthcare providers with:
- Role and specialty information
- License verification
- Approval weight for quorum calculations

## Usage Flow

1. **Registration**: Emergency authorities are registered by admin
2. **Request**: Provider submits emergency access request with MFA
3. **Approval**: Multiple authorities approve based on configuration
4. **Grant**: Automatic grant creation upon sufficient approvals
5. **Access**: Provider can access specified records during grant period
6. **Expiration**: Automatic cleanup of expired grants
7. **Audit**: Comprehensive logging for compliance

## Security Features

- **MFA Requirements**: Configurable multi-factor authentication
- **Time Limits**: Strict time-based access controls
- **Approval Quorum**: Multiple authority approvals required
- **Audit Trails**: Complete logging of all activities
- **Dispute Resolution**: Mechanism for challenging inappropriate access
- **Automatic Revocation**: Time-based and manual revocation capabilities

## Integration Points

- **Medical Records**: Validates record access permissions
- **Identity Registry**: Verifies provider credentials
- **Governor**: Handles complex approval workflows
- **Compliance**: Logs all access for regulatory compliance
- **Dispute Resolution**: Manages access disputes

## Configuration

The contract is configured with:
- Maximum request duration
- Minimum approvals required
- Emergency cooldown periods
- MFA requirements
- Auto-expiration settings
- Audit logging preferences

## Events

The contract emits events for:
- Contract initialization
- Authority registration
- Access requests and approvals
- Grant creation and revocation
- Access disputes
- Compliance reports
- Cleanup operations

## Error Handling

Comprehensive error handling for:
- Authorization failures
- Invalid requests
- Expired access
- MFA failures
- Quorum requirements
- Dispute states
- Compliance violations