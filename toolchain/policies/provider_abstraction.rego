package policies.provider_abstraction

# Provider Abstraction Policy - INV-6
# Enforces vendor-neutral abstraction layers for provider APIs
#
# Invariant INV-6: Detect direct vendor API imports without abstraction
# This policy ensures that:
# - Vendor-specific imports (openai, anthropic, google.genai) use abstraction
# - Provider selection is decoupled from implementation
# - API calls route through standardized abstraction layer
# - No direct hardcoded vendor dependencies
# - Multiple providers remain swappable

name = "provider_abstraction"
description = "Enforce provider abstraction for vendor-neutral API usage (INV-6)"

# Rule: allow if no direct vendor API import detected
allow_if_no_direct_import = (
    input.get('direct_vendor_import', False) == False
)

# Rule: allow direct vendor import only if abstracted
allow_if_abstracted_vendor_import = (
    input.get('direct_vendor_import', False) == True and
    input.get('provider_abstracted', False) == True and
    input.get('abstraction_layer_present', False) == True
)

# Rule: deny direct openai import without abstraction
deny_direct_openai = (
    input.get('vendor_import_type', '') == 'openai' and
    input.get('provider_abstracted', False) == False
)

# Rule: deny direct anthropic import without abstraction
deny_direct_anthropic = (
    input.get('vendor_import_type', '') == 'anthropic' and
    input.get('provider_abstracted', False) == False
)

# Rule: deny direct google.genai import without abstraction
deny_direct_google_genai = (
    input.get('vendor_import_type', '') == 'google.genai' and
    input.get('provider_abstracted', False) == False
)

# Rule: require abstraction layer for vendor-specific APIs
require_abstraction_layer = (
    input.get('vendor_import_type', '') in ['openai', 'anthropic', 'google.genai'] and
    input.get('abstraction_layer_present', False) == True
)

# Rule: allow API calls through abstraction layer
allow_if_via_abstraction = (
    input.get('api_call_type', '') == 'abstracted' and
    input.get('abstraction_layer_used', False) == True
)

# Rule: deny vendor lock-in patterns
deny_hardcoded_vendor = (
    input.get('vendor_hardcoded', False) == True and
    input.get('provider_abstracted', False) == False
)

# Rule: require provider selection interface
require_provider_selection = (
    input.get('abstraction_layer_present', False) == True and
    input.get('provider_selection_interface', False) == True
)

# Rule: deny configuration with hardcoded vendor endpoint
deny_hardcoded_endpoint = (
    input.get('endpoint_hardcoded', False) == True and
    input.get('endpoint_configurable', False) == False
)

# Composite allow rule: provider abstraction requirements met
allow_operation = (
    input.get('direct_vendor_import', False) == False or
    (input.get('direct_vendor_import', False) == True and
     input.get('provider_abstracted', False) == True and
     input.get('abstraction_layer_present', False) == True and
     input.get('provider_selection_interface', False) == True)
)
