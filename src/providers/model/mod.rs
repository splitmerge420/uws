//! Model provider skeletons (Module 16A).

pub mod anthropic;
pub mod gemini;
pub mod openai;
pub mod deepseek;

#[derive(Debug, Clone)]
pub struct ModelProviderDriver {
    pub name: &'static str,
}
