#!/usr/bin/env python3
"""
Manus Sphere Adapter — Task 1.6 for Aluminum OS

This module translates Manus's ontology (used in master_ingestion.json)
to the canonical System B (Houses) ontology from verified_ontology.md.

Manus uses: house_01_mathematics, house_02_natural_sciences, etc.
Canonical System B uses: House 01 (Formal Sciences), House 02 (Natural Sciences), etc.

The sphere IDs (S001-S144) remain consistent across the mapping.
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple


# CANONICAL HOUSES - System B from verified_ontology.md
CANONICAL_HOUSES = {
    "house_01": {
        "name": "Formal Sciences",
        "sphere_range": (1, 12),
        "spheres": [f"S{i:03d}" for i in range(1, 13)]
    },
    "house_02": {
        "name": "Natural Sciences",
        "sphere_range": (13, 24),
        "spheres": [f"S{i:03d}" for i in range(13, 25)]
    },
    "house_03": {
        "name": "Engineering & Technology",
        "sphere_range": (25, 36),
        "spheres": [f"S{i:03d}" for i in range(25, 37)]
    },
    "house_04": {
        "name": "Computing & Systems",
        "sphere_range": (37, 48),
        "spheres": [f"S{i:03d}" for i in range(37, 49)]
    },
    "house_05": {
        "name": "Health & Medicine",
        "sphere_range": (49, 60),
        "spheres": [f"S{i:03d}" for i in range(49, 61)]
    },
    "house_06": {
        "name": "Mind & Behavior",
        "sphere_range": (61, 72),
        "spheres": [f"S{i:03d}" for i in range(61, 73)]
    },
    "house_07": {
        "name": "Society & Governance",
        "sphere_range": (73, 84),
        "spheres": [f"S{i:03d}" for i in range(73, 85)]
    },
    "house_08": {
        "name": "Economics & Exchange",
        "sphere_range": (85, 96),
        "spheres": [f"S{i:03d}" for i in range(85, 97)]
    },
    "house_09": {
        "name": "Law & Institutions",
        "sphere_range": (97, 108),
        "spheres": [f"S{i:03d}" for i in range(97, 109)]
    },
    "house_10": {
        "name": "Culture & Meaning",
        "sphere_range": (109, 120),
        "spheres": [f"S{i:03d}" for i in range(109, 121)]
    },
    "house_11": {
        "name": "Spiritual & Metaphysics",
        "sphere_range": (121, 132),
        "spheres": [f"S{i:03d}" for i in range(121, 133)]
    },
    "house_12": {
        "name": "Integration & Meta",
        "sphere_range": (133, 144),
        "spheres": [f"S{i:03d}" for i in range(133, 145)]
    }
}


# MANUS TO CANONICAL MAPPING
# Maps Manus house names to canonical house keys
MANUS_TO_CANONICAL = {
    "house_01_mathematics": "house_01",  # Both are Formal Sciences (S001-S012)
    "house_02_natural_sciences": "house_02",  # Both are Natural Sciences (S013-S024)
    "house_03_philosophy_psychology": "house_06",  # Maps to Mind & Behavior (S061-S072)
    "house_04_engineering": "house_03",  # Maps to Engineering & Technology (S025-S036)
    "house_05_arts_culture": "house_10",  # Maps to Culture & Meaning (S109-S120)
    "house_06_medicine_health": "house_05",  # Maps to Health & Medicine (S049-S060)
    "house_07_education": "house_07",  # Maps to Society & Governance (S073-S084)
    "house_08_law_governance": "house_09",  # Maps to Law & Institutions (S097-S108)
    "house_09_economics_finance": "house_08",  # Maps to Economics & Exchange (S085-S096)
    "house_10_technology_infrastructure": "house_04",  # Maps to Computing & Systems (S037-S048)
    "house_11_communication_media": "house_11",  # Maps to Spiritual & Metaphysics (S121-S132)
    "house_12_industry_commerce": "house_12",  # Maps to Integration & Meta (S133-S144)
}


def validate_mappings() -> Tuple[int, List[str]]:
    """
    Validate the MANUS_TO_CANONICAL mapping for conflicts.

    Returns:
        Tuple of (conflict_count, conflict_messages)
    """
    conflicts = []
    conflict_count = 0

    # Check 1: All Manus houses must be mapped
    all_manus_houses = set(MANUS_TO_CANONICAL.keys())
    expected_manus_houses = {
        "house_01_mathematics",
        "house_02_natural_sciences",
        "house_03_philosophy_psychology",
        "house_04_engineering",
        "house_05_arts_culture",
        "house_06_medicine_health",
        "house_07_education",
        "house_08_law_governance",
        "house_09_economics_finance",
        "house_10_technology_infrastructure",
        "house_11_communication_media",
        "house_12_industry_commerce",
    }

    missing_manus = expected_manus_houses - all_manus_houses
    if missing_manus:
        for house in sorted(missing_manus):
            msg = f"CONFLICT: Manus house '{house}' has no mapping"
            conflicts.append(msg)
            conflict_count += 1

    # Check 2: All mapped houses must be valid canonical houses
    for manus_house, canonical_house in MANUS_TO_CANONICAL.items():
        if canonical_house not in CANONICAL_HOUSES:
            msg = f"CONFLICT: '{manus_house}' maps to invalid canonical house '{canonical_house}'"
            conflicts.append(msg)
            conflict_count += 1

    # Check 3: Validate sphere ranges
    # Build a mapping of canonical house to its actual sphere IDs
    sphere_to_canonical = {}
    for canonical_key, canonical_data in CANONICAL_HOUSES.items():
        for sphere_id in canonical_data["spheres"]:
            if sphere_id in sphere_to_canonical:
                msg = f"CONFLICT: Sphere {sphere_id} appears in multiple canonical houses"
                conflicts.append(msg)
                conflict_count += 1
            sphere_to_canonical[sphere_id] = canonical_key

    # Check 4: All 144 spheres must be mapped exactly once
    expected_spheres = {f"S{i:03d}" for i in range(1, 145)}
    actual_spheres = set(sphere_to_canonical.keys())

    missing_spheres = expected_spheres - actual_spheres
    if missing_spheres:
        msg = f"CONFLICT: {len(missing_spheres)} spheres are not mapped in canonical houses"
        conflicts.append(msg)
        conflict_count += 1

    duplicate_spheres = []
    for sphere in actual_spheres:
        count = 0
        for house_data in CANONICAL_HOUSES.values():
            count += house_data["spheres"].count(sphere)
        if count > 1:
            duplicate_spheres.append(sphere)

    if duplicate_spheres:
        msg = f"CONFLICT: {len(duplicate_spheres)} spheres appear multiple times"
        conflicts.append(msg)
        conflict_count += 1

    return conflict_count, conflicts


def translate_ingestion(ingestion_path: str) -> Dict:
    """
    Read a Manus ingestion file and translate it to canonical System B ontology.

    Args:
        ingestion_path: Path to master_ingestion.json

    Returns:
        Dictionary with translated ingestion data using canonical house names
    """
    with open(ingestion_path, 'r') as f:
        data = json.load(f)

    translated = {
        "version": data.get("version", "1.0.0"),
        "generated": data.get("generated", ""),
        "description": "Canonical System B Ingestion Map — Translated from Manus ontology",
        "notes": "This map has been translated from Manus ontology to canonical System B (Houses) ontology",
        "sphere_houses": {}
    }

    # Translate each Manus house to canonical format
    for manus_house_name, manus_data in data.get("sphere_houses", {}).items():
        canonical_house_key = MANUS_TO_CANONICAL.get(manus_house_name)

        if not canonical_house_key:
            print(f"WARNING: Manus house '{manus_house_name}' has no canonical mapping", file=sys.stderr)
            continue

        canonical_data = CANONICAL_HOUSES[canonical_house_key]

        # Build translated house entry
        translated_house = {
            "canonical_name": canonical_data["name"],
            "spheres": canonical_data["spheres"],
            "repos": manus_data.get("repos", {})
        }

        translated["sphere_houses"][canonical_house_key] = translated_house

    # Copy unmapped repos and statistics
    if "unmapped_repos" in data:
        translated["unmapped_repos"] = data["unmapped_repos"]

    if "statistics" in data:
        translated["statistics"] = data["statistics"]

    return translated


def validate_cli():
    """
    CLI mode for validation. Checks for mapping conflicts and prints a report.

    Returns:
        Exit code (0 for success, 1 for conflicts found)
    """
    print("=" * 80)
    print("MANUS SPHERE ADAPTER — VALIDATION REPORT")
    print("=" * 80)
    print()

    # Validate mappings
    conflict_count, conflicts = validate_mappings()

    print(f"Total Conflicts Found: {conflict_count}")
    print()

    if conflicts:
        print("DETAILED CONFLICTS:")
        print("-" * 80)
        for conflict in conflicts:
            print(f"  • {conflict}")
        print()
    else:
        print("✓ All validations passed!")
        print()

    # Print mapping summary
    print("MANUS → CANONICAL HOUSE MAPPING:")
    print("-" * 80)
    for manus_house in sorted(MANUS_TO_CANONICAL.keys()):
        canonical_house = MANUS_TO_CANONICAL[manus_house]
        canonical_name = CANONICAL_HOUSES[canonical_house]["name"]
        print(f"  {manus_house:40} → {canonical_house:10} ({canonical_name})")
    print()

    # Print canonical houses summary
    print("CANONICAL HOUSES (System B):")
    print("-" * 80)
    for house_key in sorted(CANONICAL_HOUSES.keys()):
        house_data = CANONICAL_HOUSES[house_key]
        start, end = house_data["sphere_range"]
        print(f"  {house_key}: {house_data['name']:30} (S{start:03d}-S{end:03d})")
    print()

    # Print statistics
    print("STATISTICS:")
    print("-" * 80)
    print(f"  Total Manus Houses: {len(MANUS_TO_CANONICAL)}")
    print(f"  Total Canonical Houses: {len(CANONICAL_HOUSES)}")
    print(f"  Total Spheres: 144 (S001-S144)")
    print(f"  Mapping Conflicts: {conflict_count}")
    print()

    print("=" * 80)

    return 0 if conflict_count == 0 else 1


def main():
    """Main entry point."""
    if len(sys.argv) > 1 and sys.argv[1] == "--validate":
        exit_code = validate_cli()
        sys.exit(exit_code)
    elif len(sys.argv) > 1 and sys.argv[1] == "--help":
        print("MANUS SPHERE ADAPTER")
        print()
        print("Usage:")
        print(f"  {sys.argv[0]} --validate          Run validation and print report")
        print(f"  {sys.argv[0]} --help              Show this help message")
        print(f"  {sys.argv[0]} <ingestion_path>    Translate ingestion file to canonical format")
        print()
        sys.exit(0)
    elif len(sys.argv) > 1:
        ingestion_path = sys.argv[1]
        translated = translate_ingestion(ingestion_path)
        print(json.dumps(translated, indent=2))
    else:
        print("ERROR: Please provide --validate, --help, or a path to master_ingestion.json")
        sys.exit(1)


if __name__ == "__main__":
    main()
