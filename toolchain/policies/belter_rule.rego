package policies.belter_rule

# Belter Rule Policy - INV-30
# Enforces structured logging for operational observability
#
# Invariant INV-30: Enforce structured logging, deny print-based logging
# This policy ensures that:
# - All logging uses structured formats (structured, json, syslog)
# - Print statements and ad-hoc logging are blocked
# - Logs are parseable and queryable
# - Operational visibility is maintained
# - Debugging doesn't bypass logging standards

name = "belter_rule"
description = "Enforce structured logging and deny print-based logging (INV-30)"

# Rule: allow if using structured logging
allow_if_structured_logging = (
    input.get('logging_type', '') in ['structured', 'json', 'syslog']
)

# Rule: allow if no logging type specified (skip check)
allow_if_no_logging = input.get('logging_type', '') == ''

# Rule: deny print-based logging
deny_print_logging = input.get('logging_type', '') == 'print'

# Rule: deny unstructured logging types
deny_unstructured_logging = (
    input.get('logging_type', '') not in ['structured', 'json', 'syslog', '']
)

# Rule: require structured format for sensitive operations
require_structured_sensitive = (
    input.get('operation_sensitivity', '') in ['high', 'critical'] and
    input.get('logging_type', '') in ['structured', 'json']
)

# Rule: require logging for external calls
require_logging_external_calls = (
    input.get('has_external_calls', False) == True and
    input.get('logging_type', '') in ['structured', 'json', 'syslog']
)

# Rule: allow json logging for audit trails
allow_json_audit = (
    input.get('logging_type', '') == 'json' and
    input.get('is_audit_log', False) == True
)

# Rule: allow syslog for system events
allow_syslog_system = (
    input.get('logging_type', '') == 'syslog' and
    input.get('is_system_event', False) == True
)

# Rule: deny console.log equivalents
deny_console_logging = (
    input.get('logging_method', '') in ['print', 'console.log', 'printf'] and
    input.get('logging_type', '') == 'print'
)

# Rule: require log level specification for structured logs
require_log_level = (
    input.get('logging_type', '') in ['structured', 'json', 'syslog'] and
    input.get('log_level_specified', False) == True
)

# Composite allow rule: structured logging requirements met
allow_operation = (
    input.get('logging_type', '') in ['structured', 'json', 'syslog'] or
    input.get('logging_type', '') == ''
)
