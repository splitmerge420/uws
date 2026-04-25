//! Cloud / hyperscaler skeletons (Module 16B).

pub mod aws;
pub mod azure;
pub mod gcp;

#[derive(Debug, Clone)]
pub struct CloudProviderDriver {
    pub name: &'static str,
}
