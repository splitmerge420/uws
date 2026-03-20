package policies.consent_enforcement

name = "consent_enforcement"
description = "Enforce user consent for state-changing operations (INV-2)"

# Default-deny posture (INV-35)
default allow = false

allow_if_read_only = input.get('operation_type', '') == 'read'

allow_if_state_change_with_consent = (
    input.get('operation_type', '') in ['write', 'delete'] and
    input.get('user_consent', False) == True
)

require_deletion_consent_reason = (
    input.get('operation_type', '') == 'delete' and
    input.get('deletion_reason', '') != ''
)

deny_state_change_no_consent = (
    input.get('operation_type', '') in ['write', 'delete'] and
    input.get('user_consent', False) == False
)

require_consent_token = (
    input.get('is_sensitive_operation', False) == True and
    input.get('consent_token', None) is not None
)

allow_operation = (
    input.get('operation_type', '') == 'read' or
    (input.get('user_consent', False) == True and
     input.get('operation_type', '') in ['write', 'delete'])
)
