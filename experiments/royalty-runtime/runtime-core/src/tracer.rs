use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PackageMeta {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CanonicalLineage {
    pub primary_package: PackageMeta,
    pub runtime: PackageMeta,
    pub lockfile_digest: String,
    pub resolved_dependencies: Vec<Dependency>,
}

impl CanonicalLineage {
    pub fn generate_hash(&self) -> String {
        let canonical_string = serde_json::to_string(self)
            .expect("failed to serialize canonical lineage");
        let mut hasher = Sha256::new();
        hasher.update(canonical_string.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
