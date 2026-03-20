"""
Health System Connectors for Aluminum OS
=========================================

Provides abstract base classes for integrating with various health data sources:
- HL7 FHIR R4 health records
- Wearable devices (Apple Health, Google Fit, Fitbit)
- Academic literature (PubMed)
- Clinical trials (clinicaltrials.gov)
- FDA drug information
- Legacy HL7v2 hospital systems
- Pharmacy/insurance systems
- Emergency escalation (One Medical Crisis Adapter)

All connectors are ABC stubs awaiting implementation.
"""

from abc import ABC, abstractmethod
from datetime import datetime
from typing import List, Dict, Optional, Any, Tuple
from enum import Enum


class CrisisLevel(Enum):
    """Crisis severity levels for emergency escalation."""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


class TrialStatus(Enum):
    """Clinical trial recruitment status."""
    RECRUITING = "recruiting"
    NOT_YET_RECRUITING = "not_yet_recruiting"
    ACTIVE_NOT_RECRUITING = "active_not_recruiting"
    ENROLLING_BY_INVITATION = "enrolling_by_invitation"
    COMPLETED = "completed"
    TERMINATED = "terminated"
    SUSPENDED = "suspended"


# ============================================================================
# 1. FHIR CONNECTOR (HL7 FHIR R4)
# ============================================================================

class FHIRConnector(ABC):
    """
    HL7 FHIR R4 connector for electronic health record systems.

    Handles CRUD operations on FHIR resources including patients,
    observations, conditions, medications, and care plans.

    FHIR resources follow the standard structure:
    {
        "resourceType": "Patient|Observation|Condition|...",
        "id": "resource-id",
        "meta": {"versionId": "...", "lastUpdated": "..."},
        ...resource-specific fields...
    }
    """

    @abstractmethod
    def get_patient(self, patient_id: str) -> Dict[str, Any]:
        """
        Retrieve a patient resource by ID.

        Args:
            patient_id: FHIR patient resource ID

        Returns:
            FHIR Patient resource as dictionary

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def search_patients(self, query: Dict[str, Any]) -> List[Dict[str, Any]]:
        """
        Search for patients using FHIR search parameters.

        Common query parameters:
        {
            "name": "John Doe",
            "birthdate": "1990-01-01",
            "identifier": "12345",
            "telecom": "john@example.com"
        }

        Args:
            query: Dictionary of FHIR search parameters

        Returns:
            List of FHIR Patient resources matching criteria

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def create_observation(self, patient_id: str, observation: Dict[str, Any]) -> str:
        """
        Create a new observation (lab result, vital sign, etc).

        Observation structure:
        {
            "resourceType": "Observation",
            "status": "final",
            "category": [{
                "coding": [{"system": "http://terminology.hl7.org/CodeSystem/observation-category",
                           "code": "vital-signs"}]
            }],
            "code": {"coding": [{"system": "http://loinc.org", "code": "55284-4"}]},
            "subject": {"reference": "Patient/{patient_id}"},
            "effectiveDateTime": "2026-03-19T10:00:00Z",
            "valueQuantity": {"value": 98.6, "unit": "F"}
        }

        Args:
            patient_id: FHIR patient resource ID
            observation: FHIR Observation resource

        Returns:
            Created observation resource ID

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_conditions(self, patient_id: str) -> List[Dict[str, Any]]:
        """
        Get all conditions (diagnoses) for a patient.

        Returns FHIR Condition resources with clinical codes (ICD-10, SNOMED CT).

        Args:
            patient_id: FHIR patient resource ID

        Returns:
            List of FHIR Condition resources

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_medications(self, patient_id: str) -> List[Dict[str, Any]]:
        """
        Get all medications and medication requests for a patient.

        Returns FHIR MedicationRequest and Medication resources
        including dosage, frequency, and RxNorm codes.

        Args:
            patient_id: FHIR patient resource ID

        Returns:
            List of FHIR Medication and MedicationRequest resources

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def create_care_plan(self, patient_id: str, plan: Dict[str, Any]) -> str:
        """
        Create a new care plan for a patient.

        Care plan structure:
        {
            "resourceType": "CarePlan",
            "status": "draft",
            "intent": "plan",
            "subject": {"reference": "Patient/{patient_id}"},
            "title": "Diabetes management plan",
            "activity": [{
                "detail": {
                    "kind": "ServiceRequest",
                    "code": {"coding": [{"code": "...", "system": "..."}]},
                    "status": "not-started"
                }
            }]
        }

        Args:
            patient_id: FHIR patient resource ID
            plan: FHIR CarePlan resource

        Returns:
            Created care plan resource ID

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError


# ============================================================================
# 2. WEARABLE CONNECTOR
# ============================================================================

class WearableConnector(ABC):
    """
    Connector for wearable devices and health tracking platforms.

    Integrates with Apple Health, Google Fit, Fitbit, and similar systems
    to retrieve continuous health metrics.

    Data structure:
    {
        "timestamp": "2026-03-19T10:30:00Z",
        "value": 72,
        "unit": "bpm",
        "source": "Apple Watch",
        "confidence": 0.95
    }
    """

    @abstractmethod
    def get_heart_rate(self, start: datetime, end: datetime) -> List[Dict[str, Any]]:
        """
        Get heart rate data points in specified time range.

        Returns:
            List of heart rate measurements with timestamps and values (bpm)

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_steps(self, start: datetime, end: datetime) -> List[Dict[str, Any]]:
        """
        Get step count data in specified time range.

        Returns:
            List of step measurements, may be aggregated by hour/day

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_sleep(self, start: datetime, end: datetime) -> List[Dict[str, Any]]:
        """
        Get sleep data for specified time range.

        Sleep data includes:
        {
            "timestamp": "...",
            "duration_minutes": 450,
            "sleep_stage": "deep|light|rem|awake",
            "quality_score": 0.85
        }

        Returns:
            List of sleep sessions with stage and quality information

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_blood_pressure(self, start: datetime, end: datetime) -> List[Dict[str, Any]]:
        """
        Get blood pressure readings in specified time range.

        Format:
        {
            "timestamp": "...",
            "systolic": 120,
            "diastolic": 80,
            "unit": "mmHg"
        }

        Returns:
            List of blood pressure measurements

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def sync_data(self) -> Dict[str, Any]:
        """
        Synchronize latest data from wearable device.

        Returns:
            Summary of sync operation:
            {
                "status": "success|failure",
                "records_synced": 150,
                "last_sync": "2026-03-19T10:00:00Z",
                "next_sync": "2026-03-19T11:00:00Z"
            }

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError


# ============================================================================
# 3. PUBMED CONNECTOR
# ============================================================================

class PubMedConnector(ABC):
    """
    Connector for PubMed Central biomedical literature database.

    Provides search and retrieval of peer-reviewed articles with
    citation tracking for evidence-based medicine.
    """

    @abstractmethod
    def search(self, query: str, max_results: int = 10) -> List[Dict[str, Any]]:
        """
        Search PubMed for articles matching query.

        Returns:
            List of article summaries:
            {
                "pmid": "12345678",
                "title": "...",
                "authors": ["First Author", "Second Author"],
                "journal": "Nature Medicine",
                "pub_date": "2025-06-15",
                "abstract": "...",
                "url": "https://pubmed.ncbi.nlm.nih.gov/12345678/"
            }

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_article(self, pmid: str) -> Dict[str, Any]:
        """
        Get full article metadata and abstract.

        Args:
            pmid: PubMed ID

        Returns:
            Complete article record with citations

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_citations(self, pmid: str) -> List[Dict[str, Any]]:
        """
        Get articles that cite the specified publication.

        Args:
            pmid: PubMed ID

        Returns:
            List of citing articles

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError


# ============================================================================
# 4. CLINICAL TRIALS CONNECTOR
# ============================================================================

class ClinicalTrialsConnector(ABC):
    """
    Connector for clinicaltrials.gov database.

    Searches and tracks clinical trial opportunities with patient
    eligibility assessment.
    """

    @abstractmethod
    def search_trials(self, condition: str, status: str = "recruiting") -> List[Dict[str, Any]]:
        """
        Search for clinical trials by condition and recruitment status.

        Returns:
            List of trial summaries:
            {
                "nct_id": "NCT04552898",
                "title": "...",
                "condition": "Type 2 Diabetes",
                "status": "recruiting",
                "phase": "Phase 3",
                "sponsor": "...",
                "locations": ["Boston, MA", "Seattle, WA"],
                "enrollment": 250,
                "eligibility": {
                    "min_age": 18,
                    "max_age": 75,
                    "gender": "All",
                    "accepts_healthy": False
                }
            }

        Args:
            condition: Medical condition to search for
            status: Trial recruitment status (recruiting, active_not_recruiting, etc.)

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_trial(self, nct_id: str) -> Dict[str, Any]:
        """
        Get detailed trial information.

        Args:
            nct_id: NCT identifier

        Returns:
            Complete trial record with protocol and contact info

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def check_eligibility(self, patient_data: Dict[str, Any], nct_id: str) -> Dict[str, Any]:
        """
        Check if patient meets trial eligibility criteria.

        Patient data should include:
        {
            "age": 55,
            "gender": "M",
            "conditions": ["Type 2 Diabetes", "Hypertension"],
            "medications": ["Metformin", "Lisinopril"],
            "exclusions": ["Pregnancy", "Kidney disease"]
        }

        Returns:
            Eligibility assessment:
            {
                "eligible": True,
                "score": 0.92,
                "matched_criteria": [...],
                "unmet_criteria": [...],
                "reasoning": "..."
            }

        Args:
            patient_data: Patient clinical summary
            nct_id: NCT identifier

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError


# ============================================================================
# 5. FDA CONNECTOR
# ============================================================================

class FDAConnector(ABC):
    """
    Connector for FDA drug information databases.

    Provides drug labeling, interaction checking, and recall information.
    """

    @abstractmethod
    def search_drugs(self, query: str) -> List[Dict[str, Any]]:
        """
        Search FDA drug database.

        Returns:
            List of drugs:
            {
                "ndc": "0069-3071-20",
                "brand_name": "Lipitor",
                "generic_name": "atorvastatin",
                "manufacturer": "Pfizer",
                "strength": "20 mg",
                "form": "Tablet",
                "route": "Oral"
            }

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_drug_label(self, ndc: str) -> Dict[str, Any]:
        """
        Get FDA-approved drug label and prescribing information.

        Label includes:
        {
            "ndc": "...",
            "indications": "...",
            "dosage": "...",
            "contraindications": [...],
            "warnings": [...],
            "adverse_reactions": [...],
            "drug_interactions": [...],
            "pregnancy_category": "..."
        }

        Args:
            ndc: National Drug Code

        Returns:
            Complete drug label data

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def check_interactions(self, drug_list: List[str]) -> List[Dict[str, Any]]:
        """
        Check for interactions among multiple drugs.

        Args:
            drug_list: List of drug names or NDC codes

        Returns:
            List of identified interactions:
            {
                "drug1": "...",
                "drug2": "...",
                "severity": "Major|Moderate|Minor",
                "mechanism": "...",
                "recommendation": "..."
            }

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def get_recalls(self, query: str) -> List[Dict[str, Any]]:
        """
        Search FDA recall database.

        Returns:
            List of recalls:
            {
                "ndc": "...",
                "drug_name": "...",
                "reason": "...",
                "recall_date": "2026-01-15",
                "recall_class": "Class I|II|III",
                "status": "Ongoing|Completed"
            }

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError


# ============================================================================
# 6. HL7v2 CONNECTOR (Legacy Hospital Systems)
# ============================================================================

class HL7v2Connector(ABC):
    """
    Connector for legacy HL7 version 2.x hospital systems.

    Handles traditional hospital messaging including admission/discharge/transfer (ADT),
    orders (ORM), and results (ORU).

    HL7v2 message segments are pipe-delimited with field separators.
    """

    @abstractmethod
    def send_adt(self, message: Dict[str, Any]) -> Dict[str, Any]:
        """
        Send ADT (Admission, Discharge, Transfer) message.

        Message structure:
        {
            "event_type": "A01|A04|A05",  # A01=Admission, A04=Register, A05=Pre-admission
            "patient_id": "12345",
            "mrn": "MRN123456",
            "name": "John Doe",
            "dob": "1990-01-01",
            "gender": "M",
            "admit_date": "2026-03-19T10:00:00Z",
            "attending_physician": "Dr. Smith",
            "department": "Cardiology"
        }

        Returns:
            Acknowledgment:
            {
                "ack_code": "AA|AE|AR",  # AA=Accept, AE=Application Error, AR=Application Reject
                "message_id": "...",
                "timestamp": "..."
            }

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def send_orm(self, message: Dict[str, Any]) -> Dict[str, Any]:
        """
        Send ORM (Order Message) for lab/imaging orders.

        Message structure:
        {
            "order_id": "ORD123456",
            "patient_id": "12345",
            "order_codes": [{"code": "86900", "description": "Urinalysis"}],
            "ordering_provider": "Dr. Smith",
            "clinical_info": "Diabetic patient, monitor glucose"
        }

        Returns:
            Order acknowledgment

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def receive_oru(self) -> List[Dict[str, Any]]:
        """
        Receive ORU (Observation Result) messages with lab results.

        Returns:
            List of result messages:
            {
                "order_id": "...",
                "patient_id": "...",
                "test_code": "86900",
                "result": "Negative",
                "reference_range": "...",
                "status": "F",  # F=Final, P=Preliminary
                "result_date": "2026-03-19T15:00:00Z"
            }

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError


# ============================================================================
# 7. PHARMACY CONNECTOR
# ============================================================================

class PharmacyConnector(ABC):
    """
    Connector for pharmacy benefit managers and insurance formularies.

    Handles prescription submissions, formulary lookups, and coverage verification.
    """

    @abstractmethod
    def check_coverage(self, patient_id: str, drug_ndc: str) -> Dict[str, Any]:
        """
        Check if drug is covered under patient's insurance plan.

        Returns:
            Coverage information:
            {
                "covered": True,
                "tier": "Tier 1|2|3|4",  # 1=Generic, 4=Non-preferred brand
                "copay": 10.00,
                "deductible_met": True,
                "prior_auth_required": False,
                "quantity_limit": None,
                "alternative_drugs": [{"ndc": "...", "tier": "1"}]
            }

        Args:
            patient_id: Patient identifier
            drug_ndc: National Drug Code

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def submit_prescription(self, prescription: Dict[str, Any]) -> str:
        """
        Submit prescription to pharmacy benefit manager.

        Prescription structure:
        {
            "patient_id": "...",
            "ndc": "0069-3071-20",
            "quantity": 30,
            "days_supply": 30,
            "daw": False,  # Dispense As Written
            "refills": 2,
            "prescriber_npi": "1234567890",
            "indication": "High cholesterol"
        }

        Returns:
            Prescription ID/receipt number

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def check_formulary(self, drug_ndc: str, plan_id: str) -> Dict[str, Any]:
        """
        Check if drug is on insurance plan's formulary.

        Returns:
            Formulary status and tier information

        Args:
            drug_ndc: National Drug Code
            plan_id: Insurance plan identifier

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError


# ============================================================================
# 8. ONE MEDICAL CRISIS ADAPTER (Amazon One Medical Emergency Escalation)
# ============================================================================

class OneMedicalCrisisAdapter(ABC):
    """
    Crisis escalation connector for Amazon One Medical integration.

    Per AHCEP specification: Handles emergency routing, patient context
    handoff, and outcome tracking for acute clinical situations.

    Manages transitions from routine care to crisis intervention with
    appropriate fallback chains when primary escalation fails.
    """

    @abstractmethod
    def check_availability(self, crisis_type: str, jurisdiction: str) -> Dict[str, Any]:
        """
        Check availability of crisis resources in jurisdiction.

        Crisis types: "chest_pain", "respiratory_distress", "altered_consciousness",
                      "severe_bleeding", "poisoning", "trauma", "stroke", "sepsis"

        Returns:
            Availability status:
            {
                "available": True,
                "wait_time_minutes": 5,
                "resource_type": "telehealth|urgent_care|ed",
                "facility": {
                    "name": "Boston Medical Center ED",
                    "address": "...",
                    "phone": "617-638-6000",
                    "capabilities": ["trauma_center", "stroke_center", "icu"]
                },
                "fallback_options": [...]
            }

        Args:
            crisis_type: Type of medical emergency
            jurisdiction: Geographic jurisdiction (city, state, zip)

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def escalate(self, patient_context: Dict[str, Any], crisis_level: str) -> Dict[str, Any]:
        """
        Escalate patient to appropriate crisis response level.

        Patient context should include:
        {
            "patient_id": "...",
            "name": "...",
            "dob": "...",
            "mrn": "...",
            "current_medications": [...],
            "allergies": [...],
            "conditions": [...],
            "vitals": {
                "heart_rate": 110,
                "blood_pressure": "160/100",
                "respiratory_rate": 24,
                "temperature": 98.6,
                "oxygen_saturation": 0.92
            },
            "chief_complaint": "...",
            "symptom_onset": "2026-03-19T09:30:00Z",
            "current_location": "Home"
        }

        Crisis levels: "low", "medium", "high", "critical"

        Returns:
            Escalation status:
            {
                "escalation_id": "ESC-20260319-001",
                "target_facility": {...},
                "estimated_handoff_time": "2026-03-19T10:15:00Z",
                "transport_arranged": True,
                "transport_eta_minutes": 8,
                "session_id": "SESSION-123456",
                "next_step": "Transport arrival; prepare patient data handoff"
            }

        Args:
            patient_context: Complete patient clinical summary
            crisis_level: Severity level triggering escalation

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def build_handoff_packet(self, clinical_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Build complete FHIR-compliant data packet for care transition.

        Clinical data includes EHR records, vital signs, medication list,
        allergies, and relevant recent notes.

        Returns:
            Handoff packet:
            {
                "packet_id": "PKT-20260319-001",
                "timestamp": "2026-03-19T10:00:00Z",
                "patient_summary": {...},
                "fhir_bundle": {...},
                "current_medications": [...],
                "recent_labs": [...],
                "imaging": [...],
                "clinical_notes": "...",
                "alerts": [...],
                "receiving_facility": {...},
                "transport_notes": "..."
            }

        Args:
            clinical_data: EHR data to include in handoff

        Returns:
            Complete handoff packet ready for transmission

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def log_outcome(self, session_id: str, outcome: Dict[str, Any]) -> None:
        """
        Log the outcome of crisis escalation for analytics and quality tracking.

        Outcome structure:
        {
            "outcome_type": "admission|discharge|transfer|deterioration",
            "disposition": "ICU|Ward|Home|Other",
            "duration_hours": 6.5,
            "procedures_performed": [...],
            "final_diagnosis": "...",
            "notes": "..."
        }

        Args:
            session_id: Escalation session identifier
            outcome: Outcome summary and results

        Returns:
            None

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError

    @abstractmethod
    def fallback_route(self, crisis_type: str) -> Dict[str, Any]:
        """
        Get fallback escalation path if primary route unavailable.

        Implements escalation chain for resilience:
        - Primary: One Medical telehealth
        - Secondary: Partner urgent care
        - Tertiary: Local emergency department
        - Final: 911/EMS dispatch

        Returns:
            Fallback routing options:
            {
                "chain": [
                    {
                        "priority": 1,
                        "method": "telehealth",
                        "provider": "One Medical",
                        "contact": "1-844-663-3666"
                    },
                    {
                        "priority": 2,
                        "method": "urgent_care",
                        "facility": {...}
                    },
                    {
                        "priority": 3,
                        "method": "emergency_department",
                        "facility": {...}
                    },
                    {
                        "priority": 4,
                        "method": "ems",
                        "number": "911"
                    }
                ],
                "decision_logic": "..."
            }

        Args:
            crisis_type: Type of medical emergency

        Returns:
            Fallback escalation chain

        Raises:
            NotImplementedError: Subclass must implement
        """
        raise NotImplementedError


# ============================================================================
# CONNECTOR REGISTRY
# ============================================================================

class HealthConnectorRegistry:
    """
    Central registry for managing health connector instances.

    Provides singleton pattern for connector management with
    lazy initialization and connector lifecycle tracking.
    """

    def __init__(self):
        """Initialize empty connector registry."""
        self._connectors: Dict[str, Any] = {}
        self._connector_types: Dict[str, type] = {
            "fhir": FHIRConnector,
            "wearable": WearableConnector,
            "pubmed": PubMedConnector,
            "clinical_trials": ClinicalTrialsConnector,
            "fda": FDAConnector,
            "hl7v2": HL7v2Connector,
            "pharmacy": PharmacyConnector,
            "one_medical_crisis": OneMedicalCrisisAdapter,
        }

    def register(self, name: str, connector_instance: Any) -> None:
        """
        Register a connector instance.

        Args:
            name: Unique connector identifier
            connector_instance: Instantiated connector (must be ABC subclass)

        Raises:
            TypeError: If connector_instance is not an ABC subclass
        """
        if not isinstance(connector_instance, ABC):
            raise TypeError(f"Connector must be ABC subclass, got {type(connector_instance)}")
        self._connectors[name] = connector_instance

    def get(self, name: str) -> Optional[Any]:
        """
        Retrieve a registered connector.

        Args:
            name: Connector identifier

        Returns:
            Connector instance or None if not registered
        """
        return self._connectors.get(name)

    def list_connectors(self) -> Dict[str, str]:
        """
        List all registered connectors with their types.

        Returns:
            Dictionary mapping connector names to their types
        """
        return {name: type(conn).__name__ for name, conn in self._connectors.items()}

    def list_available_types(self) -> List[str]:
        """
        List all available connector types.

        Returns:
            List of connector type names
        """
        return list(self._connector_types.keys())

    def is_registered(self, name: str) -> bool:
        """
        Check if a connector is registered.

        Args:
            name: Connector identifier

        Returns:
            True if connector is registered
        """
        return name in self._connectors

    def unregister(self, name: str) -> bool:
        """
        Unregister a connector.

        Args:
            name: Connector identifier

        Returns:
            True if connector was registered and removed
        """
        if name in self._connectors:
            del self._connectors[name]
            return True
        return False

    def clear(self) -> None:
        """Clear all registered connectors."""
        self._connectors.clear()


# Global registry instance
_global_registry = HealthConnectorRegistry()


def get_registry() -> HealthConnectorRegistry:
    """
    Get the global health connector registry.

    Returns:
        Global HealthConnectorRegistry instance
    """
    return _global_registry


__all__ = [
    "FHIRConnector",
    "WearableConnector",
    "PubMedConnector",
    "ClinicalTrialsConnector",
    "FDAConnector",
    "HL7v2Connector",
    "PharmacyConnector",
    "OneMedicalCrisisAdapter",
    "HealthConnectorRegistry",
    "CrisisLevel",
    "TrialStatus",
    "get_registry",
]
