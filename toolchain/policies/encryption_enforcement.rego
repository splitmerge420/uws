package policies.encryption_enforcement

# Encryption Enforcement Policy - INV-11/12
# Enforces encryption for sensitive data and network operations
#
# Invariant INV-11: Block writes of sensitive data without encryption
# Invariant INV-12: Block network operations without TLS
# This policy ensures that:
# - Confidential and restricted data writes are encrypted
# - Network operations use TLS/HTTPS
# - In-transit encryption is mandatory for sensitive data
# - Unencrypted channels are blocked for protected operations

name = "encryption_enforcement"
description = "Enforce encryption for sensitive data writes and network operations (INV-11/12)"

# Rule: allow sensitive data write if encryption enabled
allow_if_sensitive_data_encrypted = (
    input.get('data_classification', '') in ['confidential', 'restricted'] and
    input.get('operation_type', '') == 'write' and
    input.get('encryption_enabled', False) == True
)

# Rule: allow public/internal data write without encryption requirement
allow_if_low_sensitivity_write = (
    input.get('data_classification', '') in ['public', 'internal'] and
    input.get('operation_type', '') == 'write'
)

# Rule: allow network operations with TLS enabled
allow_if_network_tls_enabled = (
    input.get('operation_type', '') == 'network' and
    input.get('tls_enabled', False) == True
)

# Rule: allow read operations regardless of encryption
allow_if_read_only = input.get('operation_type', '') == 'read'

# Rule: deny sensitive data write without encryption
deny_sensitive_data_no_encryption = (
    input.get('data_classification', '') in ['confidential', 'restricted'] and
    input.get('operation_type', '') == 'write' and
    input.get('encryption_enabled', False) == False
)

# Rule: deny network operations without TLS
deny_network_no_tls = (
    input.get('operation_type', '') == 'network' and
    input.get('tls_enabled', False) == False
)

# Rule: deny encryption bypass attempts
deny_encryption_bypass = (
    input.get('bypass_encryption', False) == True and
    input.get('data_classification', '') in ['confidential', 'restricted']
)

# Rule: require encryption key management for sensitive data
require_key_management = (
    input.get('data_classification', '') in ['confidential', 'restricted'] and
    input.get('encryption_key_managed', False) == True
)

# Rule: deny weak cipher suites
deny_weak_tls = (
    input.get('operation_type', '') == 'network' and
    input.get('tls_version', '') not in ['1.2', '1.3']
)

# Composite allow rule: encryption requirements met
allow_operation = (
    input.get('operation_type', '') == 'read' or
    (input.get('data_classification', '') in ['public', 'internal'] and
     input.get('operation_type', '') == 'write') or
    (input.get('data_classification', '') in ['confidential', 'restricted'] and
     input.get('operation_type', '') == 'write' and
     input.get('encryption_enabled', False) == True) or
    (input.get('operation_type', '') == 'network' and
     input.get('tls_enabled', False) == True)
)
