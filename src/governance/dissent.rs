/// Records disagreement or override decisions in governance flows.
#[derive(Debug, Clone)]
pub struct DissentRecord {
    pub reason: String,
    pub actor: Option<String>,
}

impl DissentRecord {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            actor: None,
        }
    }
}
