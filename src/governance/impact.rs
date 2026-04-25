/// Resource usage accounting for governance decisions.
#[derive(Debug, Clone, Default)]
pub struct MetabolicImpact {
    pub power_watts: Option<f64>,
    pub water_liters: Option<f64>,
    pub heat_joules: Option<f64>,
}

/// Whether an operation should include metabolic impact accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetabolicImpactRequirement {
    NotRequired,
    Recommended,
    Required,
}

impl MetabolicImpact {
    pub fn new() -> Self {
        Self::default()
    }
}
