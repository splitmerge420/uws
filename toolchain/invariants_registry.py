#!/usr/bin/env python3
"""
invariants_registry.py — Complete Aluminum OS Constitutional Invariant Registry
Version: 1.0.0

Defines all 36 Aluminum Invariants (INV-1 through INV-36) as a Python constants
module. This is the single source of truth for invariant definitions across
the entire toolchain: linter, healer, pipeline, stress test.

System B (Houses) ontology is CANONICAL.

Author: Claude (Constitutional Scribe) for Dave Sheldon / Aluminum OS
Date: March 19, 2026
"""

import re
from typing import Dict, Any

INVARIANTS: Dict[str, Dict[str, Any]] = {
    "INV-1": {
        "name": "User Sovereignty",
        "description": "The user is the ultimate authority. No AI agent may override explicit user decisions.",
        "severity": "critical",
        "check_type": "advisory",
        "applies_to": ["*.py", "*.rs", "*.ts", "*.js"],
    },
    "INV-2": {
        "name": "Consent Gating",
        "description": "All state-changing operations require explicit consent before execution.",
        "severity": "critical",
        "check_type": "guard_check",
        "applies_to": ["*.py", "*.rs", "*.ts", "*.js"],
        "dangerous_patterns": [
            r"os\.system\s*\(",
            r"subprocess\.(run|call|Popen)\s*\(",
            r"open\s*\([^)]*['\"]w",
            r"shutil\.(rmtree|move|copy)",
            r"os\.(remove|unlink|rename|mkdir)",
            r"fs\.(writeFile|unlink|rmdir)",
            r"std::fs::(write|remove|create_dir)",
        ],
        "guard_patterns": [
            r"consent", r"authorize", r"approve", r"confirm",
            r"validate_permission", r"ConsentManager", r"consent_manager", r"auto_consent",
        ],
    },
    "INV-3": {
        "name": "Audit Trail",
        "description": "Every governance decision, data access, and state change must be logged immutably.",
        "severity": "critical",
        "check_type": "guard_check",
        "applies_to": ["*.py", "*.rs", "*.ts", "*.js"],
        "dangerous_patterns": [
            r"def\s+(delete|remove|destroy|drop|purge)\s*\(",
            r"fn\s+(delete|remove|destroy|drop|purge)\s*\(",
            r"async\s+function\s+(delete|remove|destroy)\s*\(",
        ],
        "guard_patterns": [r"audit", r"log(ger|ging)?", r"record", r"trace", r"AuditTrail", r"AuditChain"],
    },
    "INV-4": {"name": "Data Classification", "description": "All data must be classified by sensitivity before processing or storage.", "severity": "mandatory", "check_type": "advisory", "applies_to": ["*.py", "*.rs", "*.ts"]},
    "INV-5": {"name": "Constitutional Authority", "description": "Dave Protocol: Dave Sheldon has veto power on all Critical-severity rules.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-6": {
        "name": "Provider Abstraction",
        "description": "All cloud/AI provider calls must go through an abstraction layer.",
        "severity": "mandatory",
        "check_type": "guard_check",
        "applies_to": ["*.py", "*.ts", "*.js"],
        "dangerous_patterns": [r"from\s+openai\s+import", r"from\s+anthropic\s+import", r"from\s+google\s+import\s+genai", r"import\s+openai"],
        "guard_patterns": [r"provider", r"multi_provider", r"fallback", r"ProviderRouter", r"MultiProviderLLM", r"vendor_balance"],
    },
    "INV-7": {
        "name": "Vendor Balance",
        "description": "No single-vendor dependency. Every external API call must have a fallback provider.",
        "severity": "critical",
        "check_type": "guard_check",
        "applies_to": ["*.py", "*.ts", "*.js"],
        "dangerous_patterns": [r"(?:openai|anthropic|google\.genai|xai)\."],
        "guard_patterns": [r"fallback", r"alternative", r"vendor_balance", r"multi_provider", r"PROVIDERS\s*=", r"provider_chain"],
    },
    "INV-8": {"name": "Cross-Platform Compatibility", "description": "Core functionality must work across macOS, Linux, ChromeOS, iOS, and Android.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-9": {"name": "Offline Capability", "description": "Critical operations must function without network connectivity.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-10": {"name": "Interoperability", "description": "All data formats must be standards-compliant (JSON-LD, FHIR, W3C PROV, etc.).", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-11": {
        "name": "Encryption at Rest",
        "description": "All sensitive data must be encrypted at rest using AES-256 or quantum-resistant algorithms.",
        "severity": "critical",
        "check_type": "pattern_absence_negative",
        "applies_to": ["*.py", "*.rs", "*.ts"],
        "forbidden_patterns": [r"password\s*=\s*['\"][^'\"]{3,}['\"]", r"api_key\s*=\s*['\"][^'\"]{3,}['\"]", r"secret\s*=\s*['\"][^'\"]{3,}['\"]", r"token\s*=\s*['\"][^'\"]{3,}['\"]"],
    },
    "INV-12": {
        "name": "Encryption in Transit",
        "description": "All network communication must use TLS 1.3+ or equivalent.",
        "severity": "critical",
        "check_type": "pattern_absence_negative",
        "applies_to": ["*.py", "*.ts", "*.js"],
        "forbidden_patterns": [r"http://(?!localhost|127\.0\.0\.1|0\.0\.0\.0)"],
    },
    "INV-13": {"name": "Post-Quantum Readiness", "description": "Cryptographic systems must support or plan for NIST PQC (ML-KEM, ML-DSA).", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-14": {"name": "Zero-Knowledge Where Possible", "description": "Prefer zero-knowledge proofs for identity verification and data sharing.", "severity": "advisory", "check_type": "advisory", "applies_to": []},
    "INV-15": {"name": "Key Rotation", "description": "All cryptographic keys must have rotation schedules and automated rotation support.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-16": {"name": "Data Minimization", "description": "Collect only the minimum data necessary for the operation.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-17": {"name": "Right to Delete", "description": "Users must be able to delete all their data, including backups, within 72 hours.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-18": {"name": "Data Portability", "description": "Users must be able to export all their data in standard formats.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-19": {"name": "Jurisdictional Compliance", "description": "Data storage and processing must comply with jurisdictional requirements (GDPR, CCPA, HIPAA, etc.).", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-20": {"name": "No Silent Sharing", "description": "No data may be shared with third parties without explicit, informed, revocable consent.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-21": {
        "name": "Error Boundaries",
        "description": "Every module must have error boundaries that prevent cascade failures.",
        "severity": "mandatory",
        "check_type": "guard_check",
        "applies_to": ["*.py"],
        "dangerous_patterns": [r"except\s*:"],
        "guard_patterns": [r"except\s+\w+", r"except\s+\("],
    },
    "INV-22": {"name": "Type Safety", "description": "All public interfaces must have type annotations.", "severity": "warning", "check_type": "advisory", "applies_to": ["*.py", "*.ts"]},
    "INV-23": {"name": "Test Coverage", "description": "All critical paths must have unit tests. Target 80%+ coverage on governance code.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-24": {"name": "Graceful Degradation", "description": "Systems must degrade gracefully under failure rather than crash entirely.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-25": {"name": "Observability", "description": "All systems must expose health metrics, structured logs, and distributed traces.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-26": {"name": "Noosphere Sovereignty", "description": "User data sovereignty enforced via decentralized storage (IPFS/Noosphere).", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-27": {"name": "Session Continuity", "description": "All AI sessions must persist context across restarts.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-28": {"name": "Reincarnation Readiness", "description": "Architecture must support cold-start context restoration from persistent memory.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
    "INV-29": {"name": "Kintsugi Healing", "description": "All code changes must be tracked as fracture-mend pairs with beauty scores.", "severity": "warning", "check_type": "advisory", "applies_to": []},
    "INV-30": {
        "name": "Belter Rule",
        "description": "No structured logging bypass. All operations must use structured logging, not print statements.",
        "severity": "warning",
        "check_type": "pattern_absence_negative",
        "applies_to": ["*.py"],
        "forbidden_patterns": [r"^\s*print\s*\("],
    },
    "INV-31": {"name": "Crisis Sovereignty", "description": "During active crisis, user safety overrides all other system priorities.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-31a": {"name": "Crisis Consent Override", "description": "In life-threatening situations, system may bypass normal consent for emergency contact routing.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-31b": {"name": "Crisis Data Isolation", "description": "All crisis interaction data stored in separate HIPAA-compliant partition.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-32": {"name": "Health-Commerce Separation", "description": "Health data and commerce data must never share a processing pipeline.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-32a": {"name": "Clinical Handoff Integrity", "description": "Handoff packet to licensed professional contains only clinically relevant data.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-33": {"name": "Union-Set Jurisdiction", "description": "Apply ALL applicable laws simultaneously, not just the strictest single one.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-34": {"name": "Multi-Vantage Jurisdiction Detection", "description": "Jurisdiction detection must cross-reference 4+ signals.", "severity": "critical", "check_type": "advisory", "applies_to": []},
    "INV-35": {
        "name": "Hard Fail-Closed",
        "description": "On timeout or ambiguity: Class A data shred, Class B hold-and-notify, Class C encrypted cache.",
        "severity": "critical",
        "check_type": "guard_check",
        "applies_to": ["*.py", "*.ts", "*.rs"],
        "dangerous_patterns": [r"timeout.*default", r"except.*pass$", r"catch.*\{\s*\}", r"on_error.*continue"],
        "guard_patterns": [r"fail.?close", r"shred", r"hold.*notify", r"encrypted.*cache", r"TimeoutHandler", r"data_classification"],
    },
    "INV-36": {"name": "Technical Invariant Enforcement", "description": "Replace natural-language principles with enforceable behavioral specs.", "severity": "mandatory", "check_type": "advisory", "applies_to": []},
}

# Canonical alias — the test suite and external tooling use CONSTITUTIONAL_INVARIANTS
CONSTITUTIONAL_INVARIANTS = list(
    {"id": k, **v} for k, v in INVARIANTS.items()
)

def get_invariant(inv_id: str) -> Dict[str, Any]:
    return INVARIANTS.get(inv_id, {})

def get_invariants_by_severity(severity: str) -> Dict[str, Dict[str, Any]]:
    return {k: v for k, v in INVARIANTS.items() if v.get("severity") == severity}

def get_checkable_invariants() -> Dict[str, Dict[str, Any]]:
    return {k: v for k, v in INVARIANTS.items() if v.get("check_type") != "advisory"}

def get_invariant_count() -> int:
    return len(INVARIANTS)

def validate_registry() -> bool:
    required_fields = {"name", "description", "severity", "check_type"}
    for inv_id, inv in INVARIANTS.items():
        missing = required_fields - set(inv.keys())
        if missing:
            print(f"ERROR: {inv_id} missing fields: {missing}")
            return False
    return True

if __name__ == "__main__":
    import sys, json
    if "--validate" in sys.argv:
        valid = validate_registry()
        print(f"Registry valid: {valid}, Total: {get_invariant_count()}, Checkable: {len(get_checkable_invariants())}")
        sys.exit(0 if valid else 1)
    elif "--json" in sys.argv:
        print(json.dumps(INVARIANTS, indent=2))
    elif "--list" in sys.argv:
        for inv_id, inv in sorted(INVARIANTS.items()):
            ck = "Y" if inv["check_type"] != "advisory" else " "
            print(f"  [{ck}] {inv_id:8s} [{inv['severity']:9s}] {inv['name']}")
    else:
        print(f"Aluminum OS Invariant Registry v1.0 — {get_invariant_count()} invariants")
