package policies.error_boundaries

# Error Boundaries Policy - INV-21
# Enforces explicit error handling with typed exceptions
#
# Invariant INV-21: Require specific exception types, not bare except
# This policy ensures that:
# - All exceptions are explicitly typed (not bare except clauses)
# - Modules with external calls have error boundaries
# - Error handling is specific and intentional
# - Unexpected errors propagate safely
# - Error swallowing is prevented

name = "error_boundaries"
description = "Enforce explicit error handling and boundaries (INV-21)"

# Rule: allow if no bare except clauses
allow_if_no_bare_except = input.get('bare_except', False) == False

# Rule: allow if error boundary present for external calls
allow_if_error_boundary_present = (
    input.get('has_external_calls', False) == True and
    input.get('error_boundary_present', False) == True
)

# Rule: deny bare except clause
deny_bare_except = input.get('bare_except', False) == True

# Rule: deny missing error boundary for modules with external calls
deny_missing_error_boundary_external = (
    input.get('has_external_calls', False) == True and
    input.get('error_boundary_present', False) == False
)

# Rule: require specific exception types
require_specific_exception_types = (
    input.get('exception_typing', '') in ['typed', 'explicit'] and
    input.get('exception_specificity', 'generic') != 'generic'
)

# Rule: allow if exceptions are specifically typed
allow_if_exceptions_typed = (
    input.get('exception_specificity', 'generic') in ['specific', 'typed'] and
    input.get('bare_except', False) == False
)

# Rule: require error handling for critical operations
require_error_handling_critical = (
    input.get('operation_criticality', '') == 'critical' and
    input.get('error_boundary_present', False) == True
)

# Rule: require logging for caught exceptions
require_exception_logging = (
    input.get('exception_caught', False) == True and
    input.get('exception_logged', False) == True
)

# Rule: deny exception swallowing without logging
deny_swallowed_exception_unlogged = (
    input.get('exception_caught', False) == True and
    input.get('exception_logged', False) == False and
    input.get('exception_rethrown', False) == False
)

# Rule: require exception context preservation
require_exception_context = (
    input.get('exception_caught', False) == True and
    input.get('exception_context_preserved', False) == True
)

# Composite allow rule: error boundary requirements met
allow_operation = (
    input.get('bare_except', False) == False and
    (input.get('has_external_calls', False) == False or
     input.get('error_boundary_present', False) == True) and
    (input.get('exception_specificity', 'generic') in ['specific', 'typed'] or
     input.get('exception_typing', '') == 'explicit')
)
