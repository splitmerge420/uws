package policies.vendor_balance

# Vendor Balance Policy - INV-7
# Prevents single-vendor lock-in and enforces provider diversity
#
# Invariant INV-7: No single-vendor dependency
# This policy ensures that:
# - Operations using provider integrations require at least 2 providers
# - Single provider setup requires fallback configuration
# - Vendor lock-in is prevented through diversity enforcement
# - Multi-region or multi-cloud deployments are preferred

name = "vendor_balance"
description = "Prevent single-vendor dependency and enforce provider diversity (INV-7)"

# Rule: allow if provider count is adequate (2 or more)
allow_if_multi_provider = input.get('provider_count', 0) >= 2

# Rule: allow if single provider has fallback configured
allow_if_fallback_configured = (
    input.get('provider_count', 0) == 1 and
    input.get('fallback_configured', False) == True
)

# Rule: allow if provider is abstracted and swappable
allow_if_provider_abstracted = (
    input.get('provider_abstracted', False) == True and
    input.get('provider_swappable', False) == True
)

# Rule: deny if single provider without fallback
deny_single_vendor_no_fallback = (
    input.get('provider_count', 0) == 1 and
    input.get('fallback_configured', False) == False
)

# Rule: deny if provider is hardcoded
deny_hardcoded_provider = (
    input.get('provider_hardcoded', False) == True and
    input.get('provider_abstracted', False) == False
)

# Rule: require documented fallback strategy for critical operations
require_fallback_strategy_critical = (
    input.get('operation_criticality', '') == 'critical' and
    input.get('fallback_strategy_documented', False) == True
)

# Rule: deny operations with undocumented single-provider setup
deny_undocumented_single_vendor = (
    input.get('provider_count', 0) == 1 and
    input.get('fallback_documented', False) == False
)

# Composite allow rule: vendor balance requirements met
allow_operation = (
    (input.get('provider_count', 0) >= 2) or
    (input.get('provider_count', 0) == 1 and input.get('fallback_configured', False) == True) or
    (input.get('provider_abstracted', False) == True and input.get('provider_swappable', False) == True)
)
