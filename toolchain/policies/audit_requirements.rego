package policies.audit_requirements

name = "audit_requirements"
description = "Enforce audit logging for destructive operations (INV-3)"

allow_deletion_with_audit = (
    input.get('operation_type', '') == 'delete' and
    input.get('audit_logging_enabled', False) == True
)

require_audit_log_entry = (
    input.get('operation_type', '') in ['delete', 'modify', 'archive'] and
    input.get('audit_log_id', None) is not None
)

require_user_identity_in_audit = (
    input.get('audit_logging_enabled', False) == True and
    input.get('user_id', None) is not None
)

require_deletion_reason = (
    input.get('operation_type', '') == 'delete' and
    input.get('deletion_reason', '') != ''
)

require_audit_retention = (
    input.get('operation_type', '') == 'delete' and
    input.get('audit_retention_days', 0) >= 365
)

deny_deletion_no_audit = (
    input.get('operation_type', '') == 'delete' and
    input.get('audit_logging_enabled', False) == False
)

require_audit_evidence_preserved = (
    input.get('operation_type', '') in ['delete', 'modify'] and
    input.get('backup_before_operation', False) == True
)

verify_audit_immutability = (
    input.get('audit_logging_enabled', False) == True and
    input.get('audit_logs_immutable', False) == True
)

allow_deletion = (
    input.get('operation_type', '') != 'delete' or
    (
        input.get('audit_logging_enabled', False) == True and
        input.get('deletion_timestamp', None) is not None and
        input.get('user_id', None) is not None and
        input.get('deletion_reason', '') != '' and
        input.get('audit_retention_days', 0) >= 365 and
        input.get('backup_before_operation', False) == True
    )
)
