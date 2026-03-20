package policies.fail_closed

# Fail Closed Policy - INV-35
# Enforces secure-by-default timeout and ambiguity handling
#
# Invariant INV-35: On timeout or ambiguity, enforce class-based handling
# This policy ensures that:
# - Class A (credentials/health): shred sensitive data on failure
# - Class B (code/configs): hold and notify on failure
# - Class C (docs/comments): cache with encryption on failure
# - Never defaults to "release" or "continue" on uncertainty
# - Fail-safe defaults protect user security

name = "fail_closed"
description = "Enforce fail-closed strategy with class-based timeout handling (INV-35)"

# Rule: allow if no timeout/ambiguity condition
allow_if_definitive = (
    input.get('operation_state', '') in ['complete', 'success'] and
    input.get('timeout_occurred', False) == False and
    input.get('ambiguous_state', False) == False
)

# Rule: allow Class A handling - shred on failure
allow_if_class_a_shred = (
    input.get('data_class', '') == 'A' and
    input.get('failure_handling_strategy', '') == 'shred' and
    input.get('operation_state', '') == 'timeout'
)

# Rule: allow Class B handling - hold and notify
allow_if_class_b_hold = (
    input.get('data_class', '') == 'B' and
    input.get('failure_handling_strategy', '') == 'hold_and_notify' and
    input.get('operation_state', '') in ['timeout', 'ambiguous']
)

# Rule: allow Class C handling - encrypted cache
allow_if_class_c_cache = (
    input.get('data_class', '') == 'C' and
    input.get('failure_handling_strategy', '') == 'encrypted_cache' and
    input.get('encryption_enabled', False) == True and
    input.get('operation_state', '') in ['timeout', 'ambiguous']
)

# Rule: deny any timeout handling that defaults to release
deny_default_release_on_timeout = (
    input.get('operation_state', '') in ['timeout', 'ambiguous'] and
    input.get('failure_handling_strategy', '') in ['release', 'continue']
)

# Rule: deny continue on ambiguity without explicit override
deny_continue_on_ambiguity = (
    input.get('ambiguous_state', False) == True and
    input.get('failure_handling_strategy', '') == 'continue' and
    input.get('explicit_ambiguity_override', False) == False
)

# Rule: require notification for held operations
require_notification_for_held = (
    input.get('failure_handling_strategy', '') == 'hold_and_notify' and
    input.get('notification_sent', False) == True
)

# Rule: require data destruction for shred
require_shred_confirmation = (
    input.get('failure_handling_strategy', '') == 'shred' and
    input.get('data_destroyed', False) == True
)

# Rule: deny missing failure handling strategy
deny_missing_failure_strategy = (
    input.get('operation_state', '') in ['timeout', 'ambiguous'] and
    input.get('failure_handling_strategy', '') == ''
)

# Composite allow rule: fail-closed requirements met
allow_operation = (
    (input.get('operation_state', '') in ['complete', 'success']) or
    (input.get('operation_state', '') == 'timeout' and
     input.get('data_class', '') == 'A' and
     input.get('failure_handling_strategy', '') == 'shred') or
    (input.get('operation_state', '') in ['timeout', 'ambiguous'] and
     input.get('data_class', '') == 'B' and
     input.get('failure_handling_strategy', '') == 'hold_and_notify') or
    (input.get('operation_state', '') in ['timeout', 'ambiguous'] and
     input.get('data_class', '') == 'C' and
     input.get('failure_handling_strategy', '') == 'encrypted_cache' and
     input.get('encryption_enabled', False) == True)
)
