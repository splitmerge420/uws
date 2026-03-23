use crate::tracer::CanonicalLineage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttributionNode {
    pub package: String,
    pub weight: f64,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttributionMap {
    pub model_version: String,
    pub weighting_strategy: String,
    pub total_weight: f64,
    pub nodes: Vec<AttributionNode>,
}

impl AttributionMap {
    pub fn calculate_v0(lineage: &CanonicalLineage) -> Self {
        let mut nodes = Vec::new();
        nodes.push(AttributionNode {
            package: lineage.primary_package.name.clone(),
            weight: 0.40,
            role: "primary".to_string(),
        });

        let dep_count = lineage.resolved_dependencies.len() as f64;
        if dep_count > 0.0 {
            let split_weight = 0.60 / dep_count;
            for dep in &lineage.resolved_dependencies {
                nodes.push(AttributionNode {
                    package: dep.name.clone(),
                    weight: split_weight,
                    role: "dependency".to_string(),
                });
            }
        }

        Self {
            model_version: "0.1".to_string(),
            weighting_strategy: "primary_40_equal_split_60".to_string(),
            total_weight: 1.0,
            nodes,
        }
    }
}
