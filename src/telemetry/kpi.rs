// src/telemetry/kpi.rs
// Net-Positive Flourishing Metric (NPFM) — Aluminum OS KPI Layer
//
// Measures whether an operation, workflow, or deployment creates net-positive
// outcomes for human flourishing. Negative or zero scores block execution.
//
// Three sub-metrics compose the NetPositiveScore:
//   1. AntiBusyworkFactor — weighted sum of job tiers; must be > 0 to unblock
//   2. KnowledgeExpansionScore — credit for educational / skill-building outcomes
//   3. ProvenancePayoutScore — credit for fair attribution and royalty routing
//
// Constitutional reference: Fiduciary Duty Against Busywork (NPFM §1)

#![allow(dead_code)]

/// The tier of a job or task, used to weight its contribution to the NPFM.
///
/// Positive tiers increase the anti-busywork factor; the `BusyworkAdministrative`
/// tier applies a negative multiplier and drags the score toward zero.
#[derive(Debug, Clone, PartialEq)]
pub enum JobTier {
    /// Humans remain in the loop with meaningful decision authority.
    /// Multiplier: 1.0×
    HighAgencyOversight,

    /// Original creative or inventive output: design, art, research, writing.
    /// Multiplier: 1.2×
    CreativeGenesis,

    /// Physical-world or spatial engineering: hardware, robotics, XR, construction.
    /// Multiplier: 1.5×
    PhysicalMetaverseEngineering,

    /// Repetitive administrative or clerical work with low human agency.
    /// Multiplier: −0.8× (drags score negative)
    BusyworkAdministrative,
}

impl JobTier {
    /// The NPFM weight for this tier.
    pub fn weight(&self) -> f64 {
        match self {
            JobTier::HighAgencyOversight => 1.0,
            JobTier::CreativeGenesis => 1.2,
            JobTier::PhysicalMetaverseEngineering => 1.5,
            JobTier::BusyworkAdministrative => -0.8,
        }
    }
}

/// A single job entry contributing to the AntiBusyworkFactor.
#[derive(Debug, Clone)]
pub struct JobEntry {
    /// Descriptive label for the job (e.g. "quarterly-report-filing").
    pub label: String,
    /// The tier that classifies this job.
    pub tier: JobTier,
    /// Number of full-time-equivalent humans affected (≥ 1).
    pub fte_count: f64,
    /// When `true`, a displaced BusyworkAdministrative worker has been routed
    /// into a positive tier (HighAgencyOversight or higher). The bonus is then
    /// applied on top of the displacement credit.
    pub displaced_to_high_agency: bool,
}

/// Bonus added to the raw score when a busywork role is eliminated *and* the
/// displaced human is routed into a high-agency tier.
const ELIMINATION_BONUS: f64 = 0.5;

/// Aggregator for all job-tier contributions.
///
/// Calling [`AntiBusyworkFactor::score`] returns the weighted sum of all
/// registered jobs. If the score is ≤ 0 the system must block execution.
#[derive(Debug, Default, Clone)]
pub struct AntiBusyworkFactor {
    entries: Vec<JobEntry>,
}

impl AntiBusyworkFactor {
    /// Create an empty aggregator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a job entry.
    pub fn add(&mut self, entry: JobEntry) {
        self.entries.push(entry);
    }

    /// Compute the weighted anti-busywork score.
    ///
    /// Each entry contributes `tier.weight() × fte_count`.
    /// Entries where `displaced_to_high_agency` is `true` receive an additional
    /// [`ELIMINATION_BONUS`] per FTE on top of the regular tier weight.
    ///
    /// Returns a score in (−∞, +∞). Scores ≤ 0 must block execution.
    pub fn score(&self) -> f64 {
        self.entries.iter().fold(0.0, |acc, e| {
            let base = e.tier.weight() * e.fte_count;
            let bonus = if e.displaced_to_high_agency
                && e.tier == JobTier::BusyworkAdministrative
            {
                ELIMINATION_BONUS * e.fte_count
            } else {
                0.0
            };
            acc + base + bonus
        })
    }

    /// Returns `true` when the score is strictly positive (execution is allowed).
    pub fn is_positive(&self) -> bool {
        self.score() > 0.0
    }
}

/// Composite Net-Positive Flourishing Metric.
///
/// Combines three sub-scores into a single decision gate.
/// All three are weighted equally by default (weight = 1.0 each).
///
/// * `anti_busywork` — must be positive to allow execution at all
/// * `knowledge_expansion` — [0.0, 1.0]; credit for learning outcomes
/// * `provenance_payout` — [0.0, 1.0]; credit for fair attribution / royalties
#[derive(Debug, Clone)]
pub struct NetPositiveScore {
    pub anti_busywork: AntiBusyworkFactor,
    /// Score in [0.0, 1.0] representing educational/skill-building outcomes.
    pub knowledge_expansion: f64,
    /// Score in [0.0, 1.0] representing provenance attribution quality.
    pub provenance_payout: f64,
}

impl NetPositiveScore {
    /// Construct a new composite KPI.
    pub fn new(
        anti_busywork: AntiBusyworkFactor,
        knowledge_expansion: f64,
        provenance_payout: f64,
    ) -> Self {
        NetPositiveScore {
            anti_busywork,
            knowledge_expansion: knowledge_expansion.clamp(0.0, 1.0),
            provenance_payout: provenance_payout.clamp(0.0, 1.0),
        }
    }

    /// The composite NPFM score.
    ///
    /// Defined as the unweighted average of all three normalized sub-scores.
    /// Anti-busywork is normalized via a tanh sigmoid so that very large
    /// positive values asymptote to 1 and negative values stay negative.
    pub fn composite(&self) -> f64 {
        let abf_normalized = self.anti_busywork.score().tanh();
        (abf_normalized + self.knowledge_expansion + self.provenance_payout) / 3.0
    }

    /// Returns `true` when the NPFM is strictly positive **and** the
    /// AntiBusyworkFactor is strictly positive.
    ///
    /// Either condition failing will return `false` and must block execution.
    pub fn is_positive(&self) -> bool {
        self.anti_busywork.is_positive() && self.composite() > 0.0
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tier_entry(tier: JobTier, fte: f64) -> JobEntry {
        JobEntry {
            label: "test".into(),
            tier,
            fte_count: fte,
            displaced_to_high_agency: false,
        }
    }

    // ── JobTier weights ──────────────────────────────────────────────────────

    #[test]
    fn high_agency_oversight_weight_is_1_0() {
        assert_eq!(JobTier::HighAgencyOversight.weight(), 1.0);
    }

    #[test]
    fn creative_genesis_weight_is_1_2() {
        assert!((JobTier::CreativeGenesis.weight() - 1.2).abs() < f64::EPSILON);
    }

    #[test]
    fn physical_metaverse_engineering_weight_is_1_5() {
        assert!((JobTier::PhysicalMetaverseEngineering.weight() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn busywork_administrative_weight_is_negative_0_8() {
        assert!((JobTier::BusyworkAdministrative.weight() + 0.8).abs() < f64::EPSILON);
    }

    // ── AntiBusyworkFactor ───────────────────────────────────────────────────

    #[test]
    fn empty_abf_scores_zero_and_is_not_positive() {
        let abf = AntiBusyworkFactor::new();
        assert_eq!(abf.score(), 0.0);
        assert!(!abf.is_positive());
    }

    #[test]
    fn single_high_agency_entry_scores_positively() {
        let mut abf = AntiBusyworkFactor::new();
        abf.add(tier_entry(JobTier::HighAgencyOversight, 2.0));
        assert!((abf.score() - 2.0).abs() < f64::EPSILON);
        assert!(abf.is_positive());
    }

    #[test]
    fn single_busywork_entry_blocks_execution() {
        let mut abf = AntiBusyworkFactor::new();
        abf.add(tier_entry(JobTier::BusyworkAdministrative, 1.0));
        assert!(abf.score() < 0.0);
        assert!(!abf.is_positive());
    }

    #[test]
    fn elimination_bonus_applied_when_displaced_to_high_agency() {
        // Score without bonus: -0.8 × 1.0 = -0.8
        // Score with bonus:    (-0.8 + 0.5) × 1.0 = -0.3 (still negative; confirm value)
        let mut abf = AntiBusyworkFactor::new();
        abf.add(JobEntry {
            label: "admin-role".into(),
            tier: JobTier::BusyworkAdministrative,
            fte_count: 1.0,
            displaced_to_high_agency: true,
        });
        let score = abf.score();
        // base: -0.8, bonus: +0.5 → -0.3
        assert!((score - (-0.3)).abs() < 1e-9);
    }

    #[test]
    fn eliminating_busywork_and_routing_to_high_agency_earns_positive_bonus() {
        // 2 busywork FTEs eliminated + routed to HighAgencyOversight (1.0×)
        // busywork entries: 2 × (-0.8 + 0.5) = 2 × -0.3 = -0.6
        // high-agency entry: 2 × 1.0 = 2.0
        // total: 2.0 - 0.6 = 1.4 > 0
        let mut abf = AntiBusyworkFactor::new();
        abf.add(JobEntry {
            label: "admin-eliminated".into(),
            tier: JobTier::BusyworkAdministrative,
            fte_count: 2.0,
            displaced_to_high_agency: true,
        });
        abf.add(tier_entry(JobTier::HighAgencyOversight, 2.0));
        assert!(abf.score() > 0.0);
        assert!(abf.is_positive());
    }

    // ── NetPositiveScore ─────────────────────────────────────────────────────

    #[test]
    fn npfm_positive_when_all_sub_scores_positive() {
        let mut abf = AntiBusyworkFactor::new();
        abf.add(tier_entry(JobTier::CreativeGenesis, 1.0));
        let npfm = NetPositiveScore::new(abf, 0.8, 0.9);
        assert!(npfm.is_positive());
    }

    #[test]
    fn npfm_blocked_when_abf_is_non_positive() {
        let abf = AntiBusyworkFactor::new(); // empty → score = 0
        let npfm = NetPositiveScore::new(abf, 1.0, 1.0);
        assert!(!npfm.is_positive());
    }

    #[test]
    fn npfm_blocked_when_abf_negative_despite_positive_sub_scores() {
        let mut abf = AntiBusyworkFactor::new();
        abf.add(tier_entry(JobTier::BusyworkAdministrative, 5.0)); // very negative
        let npfm = NetPositiveScore::new(abf, 1.0, 1.0);
        assert!(!npfm.is_positive());
    }

    #[test]
    fn knowledge_expansion_clamped_to_unit_interval() {
        let abf = AntiBusyworkFactor::new();
        let npfm = NetPositiveScore::new(abf, 2.0, -1.0);
        assert!(npfm.knowledge_expansion <= 1.0);
        assert!(npfm.provenance_payout >= 0.0);
    }
}
