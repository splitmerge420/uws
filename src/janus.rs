// src/janus.rs
// Aluminum OS — Janus: Multi-Model AI Omni-Router
//
// Named after the two-faced Roman god who sees all directions simultaneously,
// Janus evaluates every AI task against the current cost, latency, and
// quality profiles of available inference providers and routes the prompt to
// the optimal model automatically.
//
// Design principles:
//   - Zero hard-coded "best model" decisions.  The router is parameterised
//     by a `RoutingProfile` that the caller configures.
//   - Cost, latency, and quality weights are independently tunable.
//   - The router produces an ordered `RouteDecision` — primary model +
//     fallback chain — so the caller can try alternatives on failure.
//   - Model capability data (cost/latency/quality) is kept in a
//     `ModelRegistry` that can be updated at runtime from a live pricing
//     feed or a local config file.
//   - No network calls here: the router is a pure scoring function.
//
// Integration:
//   - The caller fetches the current `ModelRegistry` snapshot (from cache or
//     a live pricing API) and passes it to `JanusRouter::route()`.
//   - The resulting `RouteDecision` tells the caller which SDK to call.
//
// Constitutional note:
//   INV-7 (Vendor Balance) — no single inference provider is privileged.
//   The router chooses the best provider for each task independently.
//
// Author: GitHub Copilot (builder) + Aluminum OS Council
// Session: 2026-03-22

#![allow(dead_code)]

// ─── Model providers ─────────────────────────────────────────────────────

/// Supported AI inference providers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelProvider {
    /// Anthropic Claude family.
    Claude,
    /// Google Gemini family.
    Gemini,
    /// xAI Grok family.
    Grok,
    /// OpenAI GPT family (full-scale models).
    Gpt4,
    /// OpenAI GPT family (mini / cost-optimised models).
    GptMini,
    /// A custom provider registered by the caller.
    Custom(String),
}

impl ModelProvider {
    pub fn as_str(&self) -> &str {
        match self {
            ModelProvider::Claude => "claude",
            ModelProvider::Gemini => "gemini",
            ModelProvider::Grok => "grok",
            ModelProvider::Gpt4 => "gpt4",
            ModelProvider::GptMini => "gpt_mini",
            ModelProvider::Custom(s) => s.as_str(),
        }
    }
}

// ─── Task characteristics ─────────────────────────────────────────────────

/// What kind of work the task requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    /// Short factual lookup / Q&A.
    FactualQuery,
    /// Long-form writing, drafting, or summarisation.
    Writing,
    /// Code generation, debugging, or review.
    Coding,
    /// Mathematical reasoning or data analysis.
    Reasoning,
    /// Creative work: fiction, poetry, ideation.
    Creative,
    /// Structured data extraction from a document.
    Extraction,
    /// Multi-turn conversation / chat.
    Chat,
    /// Image analysis (requires multimodal model).
    VisionAnalysis,
    /// Long-context document understanding (> 32K tokens).
    LongContext,
}

impl TaskType {
    /// Whether this task type requires vision capability.
    pub fn requires_vision(&self) -> bool {
        matches!(self, TaskType::VisionAnalysis)
    }

    /// Whether this task type requires a large context window.
    pub fn requires_long_context(&self) -> bool {
        matches!(self, TaskType::LongContext)
    }
}

/// Complexity estimate for the task.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskComplexity {
    /// Simple task solvable by a small / fast model.
    Low,
    /// Moderate complexity; most capable models needed.
    Medium,
    /// High complexity: deep reasoning, very large context.
    High,
}

/// Describes the characteristics of a task to route.
#[derive(Debug, Clone)]
pub struct TaskCharacteristics {
    pub task_type: TaskType,
    pub complexity: TaskComplexity,
    /// Estimated input token count.
    pub input_tokens: usize,
    /// Whether the task is time-sensitive (prefer low-latency models).
    pub latency_sensitive: bool,
}

impl TaskCharacteristics {
    pub fn new(task_type: TaskType, complexity: TaskComplexity) -> Self {
        TaskCharacteristics {
            task_type,
            complexity,
            input_tokens: 1000,
            latency_sensitive: false,
        }
    }

    pub fn with_input_tokens(mut self, n: usize) -> Self {
        self.input_tokens = n;
        self
    }

    pub fn latency_sensitive(mut self) -> Self {
        self.latency_sensitive = true;
        self
    }
}

// ─── Model capabilities ───────────────────────────────────────────────────

/// Static capability and pricing profile for one model.
#[derive(Debug, Clone)]
pub struct ModelCapabilities {
    pub provider: ModelProvider,
    /// Display name (e.g. "Claude 3 Opus", "Gemini 2.5 Pro").
    pub name: String,
    /// Input cost in USD per 1 million tokens.
    pub cost_per_1m_input_usd: f64,
    /// Output cost in USD per 1 million tokens.
    pub cost_per_1m_output_usd: f64,
    /// Median time-to-first-token in milliseconds.
    pub p50_latency_ms: u64,
    /// Quality score on a 0.0–1.0 scale (higher is better).
    /// This is a composite score derived from public benchmarks.
    pub quality_score: f64,
    /// Maximum context window size in tokens.
    pub context_window_tokens: usize,
    /// Whether the model supports image input.
    pub supports_vision: bool,
    /// Whether this model is available (e.g. API key configured).
    pub available: bool,
}

impl ModelCapabilities {
    pub fn new(
        provider: ModelProvider,
        name: impl Into<String>,
        cost_per_1m_input_usd: f64,
        cost_per_1m_output_usd: f64,
        p50_latency_ms: u64,
        quality_score: f64,
        context_window_tokens: usize,
        supports_vision: bool,
    ) -> Self {
        ModelCapabilities {
            provider,
            name: name.into(),
            cost_per_1m_input_usd,
            cost_per_1m_output_usd,
            p50_latency_ms,
            quality_score,
            context_window_tokens,
            supports_vision,
            available: true,
        }
    }

    /// Estimated USD cost for a task with the given input/output token counts.
    pub fn estimate_cost_usd(&self, input_tokens: usize, output_tokens: usize) -> f64 {
        (input_tokens as f64 / 1_000_000.0) * self.cost_per_1m_input_usd
            + (output_tokens as f64 / 1_000_000.0) * self.cost_per_1m_output_usd
    }
}

// ─── Model registry ───────────────────────────────────────────────────────

/// A snapshot of available model capabilities used by the router.
///
/// Default values reflect approximate public pricing as of 2026-03-22.
/// The caller should update this from a live pricing feed in production.
pub struct ModelRegistry {
    models: Vec<ModelCapabilities>,
}

impl ModelRegistry {
    /// Create an empty registry.
    pub fn empty() -> Self {
        ModelRegistry { models: vec![] }
    }

    /// Create a registry pre-populated with reasonable defaults for the
    /// major 2026 models.  Prices are approximate and should be overridden
    /// with live data in production.
    pub fn default_2026() -> Self {
        ModelRegistry {
            models: vec![
                ModelCapabilities::new(
                    ModelProvider::Claude,
                    "Claude 3.7 Sonnet",
                    3.0,    // $3/M input
                    15.0,   // $15/M output
                    800,    // 800 ms p50
                    0.92,   // quality
                    200_000,
                    true,
                ),
                ModelCapabilities::new(
                    ModelProvider::Gemini,
                    "Gemini 2.5 Pro",
                    1.25,
                    5.0,
                    600,
                    0.91,
                    1_000_000,
                    true,
                ),
                ModelCapabilities::new(
                    ModelProvider::Grok,
                    "Grok 3",
                    2.0,
                    10.0,
                    700,
                    0.88,
                    131_072,
                    true,
                ),
                ModelCapabilities::new(
                    ModelProvider::Gpt4,
                    "GPT-4.1",
                    2.0,
                    8.0,
                    900,
                    0.90,
                    128_000,
                    true,
                ),
                ModelCapabilities::new(
                    ModelProvider::GptMini,
                    "GPT-4.1 Mini",
                    0.4,
                    1.6,
                    300,
                    0.78,
                    128_000,
                    true,
                ),
            ],
        }
    }

    pub fn add(&mut self, model: ModelCapabilities) {
        self.models.push(model);
    }

    pub fn get(&self, provider: &ModelProvider) -> Option<&ModelCapabilities> {
        self.models.iter().find(|m| &m.provider == provider)
    }

    /// Return all available models.
    pub fn available(&self) -> Vec<&ModelCapabilities> {
        self.models.iter().filter(|m| m.available).collect()
    }
}

// ─── Routing profile ──────────────────────────────────────────────────────

/// How to balance cost, latency, and quality when routing.
/// Weights must sum to 1.0; use `RoutingProfile::validate()` to check.
#[derive(Debug, Clone)]
pub struct RoutingProfile {
    /// Weight given to minimising cost (0.0–1.0).
    pub cost_weight: f64,
    /// Weight given to minimising latency (0.0–1.0).
    pub latency_weight: f64,
    /// Weight given to maximising quality (0.0–1.0).
    pub quality_weight: f64,
}

impl RoutingProfile {
    pub fn new(cost: f64, latency: f64, quality: f64) -> Self {
        RoutingProfile {
            cost_weight: cost,
            latency_weight: latency,
            quality_weight: quality,
        }
    }

    /// Preset: optimise entirely for lowest cost.
    pub fn cheapest() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    /// Preset: optimise entirely for lowest latency.
    pub fn fastest() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    /// Preset: optimise entirely for highest quality.
    pub fn best_quality() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    /// Preset: balanced across all three dimensions.
    pub fn balanced() -> Self {
        Self::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
    }

    /// Validate that weights sum to ~1.0 (within floating-point tolerance).
    pub fn validate(&self) -> bool {
        let sum = self.cost_weight + self.latency_weight + self.quality_weight;
        (sum - 1.0).abs() < 0.001
    }
}

impl Default for RoutingProfile {
    fn default() -> Self {
        Self::balanced()
    }
}

// ─── Route decision ───────────────────────────────────────────────────────

/// The result of a routing decision.
#[derive(Debug, Clone)]
pub struct RouteDecision {
    /// The primary (highest-scored) model to use.
    pub primary: ModelProvider,
    /// The primary model's display name.
    pub primary_name: String,
    /// Routing score for the primary model (0.0–1.0).
    pub primary_score: f64,
    /// Estimated USD cost for this task on the primary model.
    pub estimated_cost_usd: f64,
    /// Ordered fallback chain (second-best, third-best, …).
    pub fallback_chain: Vec<ModelProvider>,
    /// Human-readable explanation of why this model was chosen.
    pub reasoning: String,
}

// ─── Routing error ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum RoutingError {
    /// No models are available in the registry.
    NoModelsAvailable,
    /// No model meets the required capability (e.g. vision or long context).
    NoCapableModel(String),
    /// The provided RoutingProfile weights do not sum to 1.0.
    InvalidProfile(String),
}

impl std::fmt::Display for RoutingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingError::NoModelsAvailable => write!(f, "no models available"),
            RoutingError::NoCapableModel(reason) => {
                write!(f, "no capable model: {}", reason)
            }
            RoutingError::InvalidProfile(msg) => write!(f, "invalid routing profile: {}", msg),
        }
    }
}

// ─── JanusRouter ─────────────────────────────────────────────────────────

/// The Janus multi-model omni-router.
pub struct JanusRouter<'r> {
    registry: &'r ModelRegistry,
}

impl<'r> JanusRouter<'r> {
    pub fn new(registry: &'r ModelRegistry) -> Self {
        JanusRouter { registry }
    }

    /// Route a task, returning a `RouteDecision` with primary model and
    /// fallback chain.
    pub fn route(
        &self,
        task: &TaskCharacteristics,
        profile: &RoutingProfile,
    ) -> Result<RouteDecision, RoutingError> {
        if !profile.validate() {
            return Err(RoutingError::InvalidProfile(
                "weights must sum to 1.0".to_string(),
            ));
        }

        let candidates: Vec<&ModelCapabilities> = self
            .registry
            .available()
            .into_iter()
            .filter(|m| self.meets_requirements(m, task))
            .collect();

        if candidates.is_empty() {
            if task.task_type.requires_vision() {
                return Err(RoutingError::NoCapableModel(
                    "vision capability required".to_string(),
                ));
            }
            if task.task_type.requires_long_context() {
                return Err(RoutingError::NoCapableModel(
                    "long context window required".to_string(),
                ));
            }
            return Err(RoutingError::NoModelsAvailable);
        }

        // Score each candidate.
        let mut scored: Vec<(&ModelCapabilities, f64)> = candidates
            .iter()
            .map(|m| {
                let score = self.compute_score(m, task, profile);
                (*m, score)
            })
            .collect();

        // Sort descending by score, then ascending by provider name for determinism.
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.provider.as_str().cmp(b.0.provider.as_str()))
        });

        let primary = &scored[0].0;
        let primary_score = scored[0].1;

        let estimated_output_tokens = estimate_output_tokens(task);
        let estimated_cost = primary.estimate_cost_usd(task.input_tokens, estimated_output_tokens);

        let fallback_chain: Vec<ModelProvider> =
            scored[1..].iter().map(|(m, _)| m.provider.clone()).collect();

        let reasoning = self.build_reasoning(primary, task, profile, primary_score);

        Ok(RouteDecision {
            primary: primary.provider.clone(),
            primary_name: primary.name.clone(),
            primary_score,
            estimated_cost_usd: estimated_cost,
            fallback_chain,
            reasoning,
        })
    }

    /// Whether a model meets the hard requirements of a task (capability gates).
    fn meets_requirements(&self, model: &ModelCapabilities, task: &TaskCharacteristics) -> bool {
        if task.task_type.requires_vision() && !model.supports_vision {
            return false;
        }
        // Long context tasks need a model with > 100k context.
        if task.task_type.requires_long_context() && model.context_window_tokens < 100_000 {
            return false;
        }
        // Input token count must not exceed model context window.
        if task.input_tokens > model.context_window_tokens {
            return false;
        }
        true
    }

    /// Compute a composite score for a model given a task and routing profile.
    ///
    /// Score = quality_weight * quality_score
    ///       + cost_weight    * (1 - normalised_cost)
    ///       + latency_weight * (1 - normalised_latency)
    ///
    /// Normalisation is done against the min/max across all available models.
    fn compute_score(
        &self,
        model: &ModelCapabilities,
        task: &TaskCharacteristics,
        profile: &RoutingProfile,
    ) -> f64 {
        let available = self.registry.available();

        // Normalised cost score: lower cost → higher score.
        let est_out = estimate_output_tokens(task) as f64;
        let costs: Vec<f64> = available
            .iter()
            .map(|m| {
                m.estimate_cost_usd(task.input_tokens, est_out as usize)
            })
            .collect();
        let min_cost = costs.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_cost = costs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let my_cost = model.estimate_cost_usd(task.input_tokens, est_out as usize);
        let cost_score = if (max_cost - min_cost).abs() < 1e-9 {
            1.0
        } else {
            1.0 - (my_cost - min_cost) / (max_cost - min_cost)
        };

        // Normalised latency score: lower latency → higher score.
        let latencies: Vec<u64> = available.iter().map(|m| m.p50_latency_ms).collect();
        let min_lat = *latencies.iter().min().unwrap_or(&0) as f64;
        let max_lat = *latencies.iter().max().unwrap_or(&1) as f64;
        let my_lat = model.p50_latency_ms as f64;
        let latency_score = if (max_lat - min_lat).abs() < 1e-9 {
            1.0
        } else {
            1.0 - (my_lat - min_lat) / (max_lat - min_lat)
        };

        // Quality score: higher is better (already 0–1).
        let quality_score = model.quality_score;

        // Apply complexity adjustment: for high-complexity tasks, amplify the
        // quality component by down-weighting the cost/latency savings from
        // cheaper models.
        let complexity_boost = match task.complexity {
            TaskComplexity::Low => 1.0,
            TaskComplexity::Medium => 1.1,
            TaskComplexity::High => 1.25,
        };

        profile.quality_weight * quality_score * complexity_boost
            + profile.cost_weight * cost_score
            + profile.latency_weight * latency_score
    }

    fn build_reasoning(
        &self,
        model: &ModelCapabilities,
        task: &TaskCharacteristics,
        profile: &RoutingProfile,
        score: f64,
    ) -> String {
        format!(
            "Selected {} (score: {:.3}) for {:?}/{:?} task. \
             Profile: cost={:.0}% latency={:.0}% quality={:.0}%. \
             Model cost: ${:.4}/task, latency: {}ms p50, quality: {:.0}%.",
            model.name,
            score,
            task.task_type,
            task.complexity,
            profile.cost_weight * 100.0,
            profile.latency_weight * 100.0,
            profile.quality_weight * 100.0,
            model.estimate_cost_usd(task.input_tokens, estimate_output_tokens(task)),
            model.p50_latency_ms,
            model.quality_score * 100.0,
        )
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

/// Heuristic estimate of output token count from task characteristics.
fn estimate_output_tokens(task: &TaskCharacteristics) -> usize {
    match (&task.task_type, &task.complexity) {
        (TaskType::FactualQuery, TaskComplexity::Low) => 150,
        (TaskType::FactualQuery, _) => 300,
        (TaskType::Writing, TaskComplexity::Low) => 500,
        (TaskType::Writing, TaskComplexity::Medium) => 1500,
        (TaskType::Writing, TaskComplexity::High) => 4000,
        (TaskType::Coding, TaskComplexity::Low) => 400,
        (TaskType::Coding, TaskComplexity::Medium) => 1000,
        (TaskType::Coding, TaskComplexity::High) => 2000,
        (TaskType::Reasoning, TaskComplexity::High) => 2000,
        (TaskType::Reasoning, _) => 800,
        (TaskType::Creative, _) => 2000,
        (TaskType::Extraction, _) => 500,
        (TaskType::Chat, _) => 300,
        (TaskType::VisionAnalysis, _) => 500,
        (TaskType::LongContext, _) => 1000,
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> ModelRegistry {
        ModelRegistry::default_2026()
    }

    // ── ModelCapabilities ─────────────────────────────────────────────

    #[test]
    fn test_estimate_cost_usd() {
        let model = ModelCapabilities::new(
            ModelProvider::Claude,
            "Test",
            3.0,   // $3/M input
            15.0,  // $15/M output
            800,
            0.92,
            200_000,
            true,
        );
        // 1M input + 1M output = $18
        let cost = model.estimate_cost_usd(1_000_000, 1_000_000);
        assert!((cost - 18.0).abs() < 0.001);
    }

    // ── RoutingProfile ────────────────────────────────────────────────

    #[test]
    fn test_routing_profile_validate_balanced() {
        assert!(RoutingProfile::balanced().validate());
    }

    #[test]
    fn test_routing_profile_validate_cheapest() {
        assert!(RoutingProfile::cheapest().validate());
    }

    #[test]
    fn test_routing_profile_validate_invalid() {
        let bad = RoutingProfile::new(0.5, 0.5, 0.5); // sums to 1.5
        assert!(!bad.validate());
    }

    // ── JanusRouter ───────────────────────────────────────────────────

    #[test]
    fn test_route_cheapest_selects_low_cost_model() {
        let reg = registry();
        let router = JanusRouter::new(&reg);
        let task = TaskCharacteristics::new(TaskType::FactualQuery, TaskComplexity::Low);
        let decision = router.route(&task, &RoutingProfile::cheapest()).unwrap();
        // GptMini is the cheapest at $0.4/M input + $1.6/M output.
        assert_eq!(decision.primary, ModelProvider::GptMini);
    }

    #[test]
    fn test_route_best_quality_selects_high_quality_model() {
        let reg = registry();
        let router = JanusRouter::new(&reg);
        let task = TaskCharacteristics::new(TaskType::Coding, TaskComplexity::High);
        let decision = router.route(&task, &RoutingProfile::best_quality()).unwrap();
        // Claude has quality_score=0.92, the highest in default_2026.
        assert_eq!(decision.primary, ModelProvider::Claude);
    }

    #[test]
    fn test_route_fastest_selects_low_latency_model() {
        let reg = registry();
        let router = JanusRouter::new(&reg);
        let task =
            TaskCharacteristics::new(TaskType::Chat, TaskComplexity::Low).latency_sensitive();
        let decision = router.route(&task, &RoutingProfile::fastest()).unwrap();
        // GptMini has p50=300ms, the lowest.
        assert_eq!(decision.primary, ModelProvider::GptMini);
    }

    #[test]
    fn test_route_decision_has_fallback_chain() {
        let reg = registry();
        let router = JanusRouter::new(&reg);
        let task = TaskCharacteristics::new(TaskType::Writing, TaskComplexity::Medium);
        let decision = router.route(&task, &RoutingProfile::balanced()).unwrap();
        // 5 models minus the primary = 4 fallbacks.
        assert_eq!(decision.fallback_chain.len(), 4);
        // Primary should not appear in fallback.
        assert!(!decision.fallback_chain.contains(&decision.primary));
    }

    #[test]
    fn test_route_vision_task_excludes_non_vision_models() {
        let mut reg = ModelRegistry::empty();
        // Add one vision model and one non-vision model.
        let vision_model = ModelCapabilities::new(
            ModelProvider::Claude,
            "Claude Vision",
            3.0, 15.0, 800, 0.92, 200_000, true,
        );
        let mut no_vision = ModelCapabilities::new(
            ModelProvider::GptMini,
            "GPT Mini",
            0.4, 1.6, 300, 0.78, 128_000, false,
        );
        no_vision.supports_vision = false;
        reg.add(vision_model);
        reg.add(no_vision);

        let router = JanusRouter::new(&reg);
        let task = TaskCharacteristics::new(TaskType::VisionAnalysis, TaskComplexity::Medium);
        let decision = router.route(&task, &RoutingProfile::balanced()).unwrap();
        assert_eq!(decision.primary, ModelProvider::Claude);
    }

    #[test]
    fn test_route_no_vision_model_returns_error() {
        let mut reg = ModelRegistry::empty();
        let mut no_vision = ModelCapabilities::new(
            ModelProvider::GptMini,
            "GPT Mini",
            0.4, 1.6, 300, 0.78, 128_000, false,
        );
        no_vision.supports_vision = false;
        reg.add(no_vision);

        let router = JanusRouter::new(&reg);
        let task = TaskCharacteristics::new(TaskType::VisionAnalysis, TaskComplexity::Low);
        let result = router.route(&task, &RoutingProfile::balanced());
        assert!(matches!(result, Err(RoutingError::NoCapableModel(_))));
    }

    #[test]
    fn test_route_empty_registry_returns_error() {
        let reg = ModelRegistry::empty();
        let router = JanusRouter::new(&reg);
        let task = TaskCharacteristics::new(TaskType::Chat, TaskComplexity::Low);
        let result = router.route(&task, &RoutingProfile::balanced());
        assert!(matches!(result, Err(RoutingError::NoModelsAvailable)));
    }

    #[test]
    fn test_route_invalid_profile_returns_error() {
        let reg = registry();
        let router = JanusRouter::new(&reg);
        let task = TaskCharacteristics::new(TaskType::Chat, TaskComplexity::Low);
        let bad_profile = RoutingProfile::new(0.8, 0.8, 0.8);
        let result = router.route(&task, &bad_profile);
        assert!(matches!(result, Err(RoutingError::InvalidProfile(_))));
    }

    #[test]
    fn test_route_context_too_large_excluded() {
        let mut reg = ModelRegistry::empty();
        // Model with 4k context window.
        let small_model = ModelCapabilities::new(
            ModelProvider::GptMini,
            "Small",
            0.4, 1.6, 300, 0.78, 4_096, true,
        );
        // Model with 200k context window.
        let large_model = ModelCapabilities::new(
            ModelProvider::Claude,
            "Large",
            3.0, 15.0, 800, 0.92, 200_000, true,
        );
        reg.add(small_model);
        reg.add(large_model);

        let router = JanusRouter::new(&reg);
        // Task with 100k input tokens — exceeds the small model's window.
        let task = TaskCharacteristics::new(TaskType::LongContext, TaskComplexity::High)
            .with_input_tokens(100_000);
        let decision = router.route(&task, &RoutingProfile::balanced()).unwrap();
        assert_eq!(decision.primary, ModelProvider::Claude);
    }

    #[test]
    fn test_route_reasoning_includes_model_info() {
        let reg = registry();
        let router = JanusRouter::new(&reg);
        let task = TaskCharacteristics::new(TaskType::Coding, TaskComplexity::Medium);
        let decision = router.route(&task, &RoutingProfile::balanced()).unwrap();
        assert!(!decision.reasoning.is_empty());
        assert!(decision.reasoning.contains("score:"));
    }

    #[test]
    fn test_route_high_complexity_favors_quality() {
        let reg = registry();
        let router = JanusRouter::new(&reg);
        // Use best_quality profile: complexity + quality-only weighting → Claude wins.
        let task = TaskCharacteristics::new(TaskType::Reasoning, TaskComplexity::High);
        let decision = router.route(&task, &RoutingProfile::best_quality()).unwrap();
        assert_eq!(decision.primary, ModelProvider::Claude);
    }

    #[test]
    fn test_registry_get() {
        let reg = registry();
        let claude = reg.get(&ModelProvider::Claude).unwrap();
        assert_eq!(claude.provider, ModelProvider::Claude);
        assert!(claude.quality_score > 0.9);
    }

    #[test]
    fn test_registry_available() {
        let mut reg = registry();
        // Mark one model unavailable.
        reg.models[0].available = false;
        assert_eq!(reg.available().len(), 4);
    }

    #[test]
    fn test_provider_as_str() {
        assert_eq!(ModelProvider::Claude.as_str(), "claude");
        assert_eq!(ModelProvider::GptMini.as_str(), "gpt_mini");
        assert_eq!(ModelProvider::Custom("mymodel".to_string()).as_str(), "mymodel");
    }
}
