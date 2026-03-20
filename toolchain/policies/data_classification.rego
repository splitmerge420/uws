package policies.data_classification

name = "data_classification"
description = "Enforce data classification before processing (INV-4)"

allow_if_classified = input.get('data_classification', 'unclassified') in [
    'public', 'internal', 'confidential', 'restricted'
]

deny_unclassified = input.get('data_classification', 'unclassified') == 'unclassified'

require_auth_for_internal = (
    input.get('data_classification', '') == 'internal' and
    input.get('user_authenticated', False) == True
)

require_special_role_for_confidential = (
    input.get('data_classification', '') == 'confidential' and
    input.get('user_role', '') in ['admin', 'data_steward', 'security_officer']
)

require_approval_for_restricted = (
    input.get('data_classification', '') == 'restricted' and
    input.get('data_access_approval_id', None) is not None
)

require_encryption_for_sensitive = (
    input.get('data_classification', '') in ['confidential', 'restricted'] and
    input.get('encryption_enabled', False) == True
)

allow_processing = (
    input.get('data_classification', 'unclassified') != 'unclassified' and
    (
        input.get('data_classification', '') == 'public' or
        (input.get('data_classification', '') == 'internal' and input.get('user_authenticated', False) == True) or
        (input.get('data_classification', '') == 'confidential' and input.get('user_role', '') in ['admin', 'data_steward', 'security_officer']) or
        (input.get('data_classification', '') == 'restricted' and input.get('data_access_approval_id', None) is not None)
    )
)
