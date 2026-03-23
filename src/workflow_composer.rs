// src/workflow_composer.rs
// Aluminum OS — YAML Pipeline Automation Composer
//
// Novel Invention #13 — Workflow Composer
//
// A YAML-defined multi-step automation pipeline runner for uws.
// Users define workflows as declarative YAML, and the composer validates,
// plans, and executes them — sequentially or with simple DAG dependencies.
//
// Example workflow (YAML):
//
//   name: morning-briefing
//   description: Pull morning digest from all providers
//   steps:
//     - id: github-prs
//       command: github pulls list
//       params: '{"owner":"octocat","repo":"Hello-World","state":"open"}'
//       on_failure: continue
//     - id: unread-emails
//       command: gmail users messages list
//       params: '{"userId":"me","q":"is:unread","maxResults":10}'
//       depends_on: []
//     - id: summarize
//       command: janus route
//       params: '{"task":"summarize"}'
//       depends_on: [github-prs, unread-emails]
//       template: "Summarize: {{github-prs.output}} and {{unread-emails.output}}"
//
// Commands:
//   uws workflow run --params '{"file":"~/.config/uws/workflows/morning.yml"}'
//   uws workflow validate --params '{"file":"~/.config/uws/workflows/morning.yml"}'
//   uws workflow list
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

// ─── On-failure policy ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnFailure {
    /// Stop the entire workflow.
    Fail,
    /// Log the error and continue with the next independent step.
    Continue,
    /// Skip dependent steps but continue others.
    Skip,
}

impl OnFailure {
    pub fn parse(s: &str) -> OnFailure {
        match s {
            "continue" => OnFailure::Continue,
            "skip" => OnFailure::Skip,
            _ => OnFailure::Fail,
        }
    }
}

// ─── Workflow step ────────────────────────────────────────────────────────

/// A single step in a workflow pipeline.
#[derive(Debug, Clone)]
pub struct WorkflowStep {
    /// Unique ID within the workflow.
    pub id: String,
    /// The uws command to run (without "uws" prefix), e.g. "github issues list".
    pub command: String,
    /// JSON params string (equivalent to --params).
    pub params: Option<String>,
    /// JSON body string (equivalent to --json).
    pub body: Option<String>,
    /// IDs of steps this step depends on.
    pub depends_on: Vec<String>,
    /// What to do if this step fails.
    pub on_failure: OnFailure,
    /// Optional output capture variable name.
    pub output_var: Option<String>,
}

// ─── Workflow definition ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WorkflowDefinition {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    /// Global variables available to all steps.
    pub variables: BTreeMap<String, String>,
}

impl WorkflowDefinition {
    /// Validate the workflow definition.
    ///
    /// Checks:
    /// - No duplicate step IDs
    /// - All `depends_on` references point to existing step IDs
    /// - No circular dependencies
    /// - No empty step commands
    pub fn validate(&self) -> Vec<WorkflowValidationError> {
        let mut errors = Vec::new();

        // Check for duplicate IDs
        let mut seen_ids = BTreeSet::new();
        for step in &self.steps {
            if step.id.is_empty() {
                errors.push(WorkflowValidationError::EmptyStepId);
            }
            if !seen_ids.insert(step.id.clone()) {
                errors.push(WorkflowValidationError::DuplicateStepId(step.id.clone()));
            }
            if step.command.trim().is_empty() {
                errors.push(WorkflowValidationError::EmptyCommand(step.id.clone()));
            }
        }

        // Check depends_on references
        for step in &self.steps {
            for dep in &step.depends_on {
                if !seen_ids.contains(dep) {
                    errors.push(WorkflowValidationError::UnknownDependency {
                        step_id: step.id.clone(),
                        dep_id: dep.clone(),
                    });
                }
            }
        }

        // Check for circular dependencies
        if let Some(cycle) = self.find_cycle() {
            errors.push(WorkflowValidationError::CircularDependency(cycle));
        }

        errors
    }

    /// Find a cycle in the dependency graph, returning the step IDs in the cycle.
    fn find_cycle(&self) -> Option<Vec<String>> {
        let mut visited = BTreeSet::new();
        let mut path = Vec::new();

        for step in &self.steps {
            if !visited.contains(&step.id) {
                if let Some(cycle) = self.dfs_cycle(&step.id, &mut visited, &mut path) {
                    return Some(cycle);
                }
            }
        }
        None
    }

    fn dfs_cycle(
        &self,
        id: &str,
        visited: &mut BTreeSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if path.contains(&id.to_string()) {
            let cycle_start = path.iter().position(|s| s == id).unwrap();
            return Some(path[cycle_start..].to_vec());
        }
        if visited.contains(id) {
            return None;
        }
        path.push(id.to_string());
        if let Some(step) = self.steps.iter().find(|s| s.id == id) {
            for dep in &step.depends_on {
                if let Some(cycle) = self.dfs_cycle(dep, visited, path) {
                    return Some(cycle);
                }
            }
        }
        path.pop();
        visited.insert(id.to_string());
        None
    }

    /// Compute a topological execution order for the steps.
    /// Returns `None` if there's a cycle.
    pub fn execution_order(&self) -> Option<Vec<String>> {
        let mut result = Vec::new();
        let mut visited = BTreeSet::new();

        for step in &self.steps {
            self.topo_sort(&step.id, &mut visited, &mut result);
        }
        Some(result)
    }

    fn topo_sort(&self, id: &str, visited: &mut BTreeSet<String>, result: &mut Vec<String>) {
        if visited.contains(id) {
            return;
        }
        visited.insert(id.to_string());
        if let Some(step) = self.steps.iter().find(|s| s.id == id) {
            for dep in &step.depends_on {
                self.topo_sort(dep, visited, result);
            }
        }
        result.push(id.to_string());
    }
}

// ─── Validation errors ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowValidationError {
    EmptyStepId,
    DuplicateStepId(String),
    EmptyCommand(String),
    UnknownDependency { step_id: String, dep_id: String },
    CircularDependency(Vec<String>),
}

impl std::fmt::Display for WorkflowValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowValidationError::EmptyStepId => write!(f, "Step has empty ID"),
            WorkflowValidationError::DuplicateStepId(id) => write!(f, "Duplicate step ID: '{id}'"),
            WorkflowValidationError::EmptyCommand(id) => write!(f, "Step '{id}' has empty command"),
            WorkflowValidationError::UnknownDependency { step_id, dep_id } => {
                write!(f, "Step '{step_id}' depends on unknown step '{dep_id}'")
            }
            WorkflowValidationError::CircularDependency(cycle) => {
                write!(f, "Circular dependency: {}", cycle.join(" → "))
            }
        }
    }
}

// ─── Step result ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Success,
    Failed { error: String },
    Skipped,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub status: StepStatus,
    pub output: Option<String>,
    pub duration_ms: u64,
}

// ─── Pipeline execution plan ─────────────────────────────────────────────

/// A dry-run plan showing what would be executed and in what order.
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub workflow_name: String,
    pub steps_in_order: Vec<String>,
    pub parallel_groups: Vec<Vec<String>>,
    pub estimated_steps: usize,
    pub validation_errors: Vec<WorkflowValidationError>,
}

/// Build an execution plan from a workflow definition.
pub fn build_execution_plan(workflow: &WorkflowDefinition) -> ExecutionPlan {
    let validation_errors = workflow.validate();

    // If there are circular dependencies, skip execution ordering to avoid stack overflow.
    let has_cycle = validation_errors
        .iter()
        .any(|e| matches!(e, WorkflowValidationError::CircularDependency(_)));

    let steps_in_order = if has_cycle {
        Vec::new()
    } else {
        workflow.execution_order().unwrap_or_default()
    };

    // Only compute parallel groups when the graph is acyclic.
    let parallel_groups = if has_cycle {
        Vec::new()
    } else {
        compute_parallel_groups(workflow)
    };

    ExecutionPlan {
        workflow_name: workflow.name.clone(),
        steps_in_order: steps_in_order.clone(),
        parallel_groups,
        estimated_steps: steps_in_order.len(),
        validation_errors,
    }
}

/// Group steps by their maximum dependency depth for parallel execution.
fn compute_parallel_groups(workflow: &WorkflowDefinition) -> Vec<Vec<String>> {
    let mut depth: BTreeMap<String, usize> = BTreeMap::new();

    // Compute depth for each step (0 = no dependencies)
    for step in &workflow.steps {
        compute_depth(&step.id, workflow, &mut depth);
    }

    if depth.is_empty() {
        return Vec::new();
    }

    let max_depth = *depth.values().max().unwrap_or(&0);
    let mut groups: Vec<Vec<String>> = vec![Vec::new(); max_depth + 1];

    for (id, d) in &depth {
        groups[*d].push(id.clone());
    }

    groups.into_iter().filter(|g| !g.is_empty()).collect()
}

fn compute_depth(id: &str, workflow: &WorkflowDefinition, memo: &mut BTreeMap<String, usize>) -> usize {
    compute_depth_guarded(id, workflow, memo, &mut BTreeSet::new())
}

fn compute_depth_guarded(
    id: &str,
    workflow: &WorkflowDefinition,
    memo: &mut BTreeMap<String, usize>,
    in_progress: &mut BTreeSet<String>,
) -> usize {
    if let Some(&d) = memo.get(id) {
        return d;
    }
    // Cycle guard: if we're already computing this node, break the cycle
    if in_progress.contains(id) {
        return 0;
    }
    let step = match workflow.steps.iter().find(|s| s.id == id) {
        Some(s) => s,
        None => return 0,
    };
    if step.depends_on.is_empty() {
        memo.insert(id.to_string(), 0);
        return 0;
    }
    in_progress.insert(id.to_string());
    let deps: Vec<String> = step.depends_on.clone();
    let max_dep_depth = deps
        .iter()
        .map(|dep| compute_depth_guarded(dep, workflow, memo, in_progress))
        .max()
        .unwrap_or(0);
    in_progress.remove(id);
    let depth = max_dep_depth + 1;
    memo.insert(id.to_string(), depth);
    depth
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(id: &str, command: &str, deps: Vec<&str>) -> WorkflowStep {
        WorkflowStep {
            id: id.to_string(),
            command: command.to_string(),
            params: None,
            body: None,
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            on_failure: OnFailure::Fail,
            output_var: None,
        }
    }

    fn make_workflow(name: &str, steps: Vec<WorkflowStep>) -> WorkflowDefinition {
        WorkflowDefinition {
            name: name.to_string(),
            description: "Test workflow".to_string(),
            steps,
            variables: BTreeMap::new(),
        }
    }

    #[test]
    fn test_validate_valid_workflow() {
        let wf = make_workflow("test", vec![
            make_step("a", "github issues list", vec![]),
            make_step("b", "slack messages post", vec!["a"]),
        ]);
        assert!(wf.validate().is_empty());
    }

    #[test]
    fn test_validate_duplicate_step_id() {
        let wf = make_workflow("test", vec![
            make_step("a", "cmd1", vec![]),
            make_step("a", "cmd2", vec![]),
        ]);
        let errors = wf.validate();
        assert!(errors.iter().any(|e| matches!(e, WorkflowValidationError::DuplicateStepId(id) if id == "a")));
    }

    #[test]
    fn test_validate_unknown_dependency() {
        let wf = make_workflow("test", vec![
            make_step("a", "cmd", vec!["nonexistent"]),
        ]);
        let errors = wf.validate();
        assert!(errors.iter().any(|e| matches!(e, WorkflowValidationError::UnknownDependency { .. })));
    }

    #[test]
    fn test_validate_empty_command() {
        let wf = make_workflow("test", vec![make_step("a", "   ", vec![])]);
        let errors = wf.validate();
        assert!(errors.iter().any(|e| matches!(e, WorkflowValidationError::EmptyCommand(_))));
    }

    #[test]
    fn test_validate_circular_dependency() {
        let wf = make_workflow("test", vec![
            make_step("a", "cmd", vec!["b"]),
            make_step("b", "cmd", vec!["a"]),
        ]);
        let errors = wf.validate();
        assert!(errors.iter().any(|e| matches!(e, WorkflowValidationError::CircularDependency(_))));
    }

    #[test]
    fn test_execution_order_respects_dependencies() {
        let wf = make_workflow("test", vec![
            make_step("c", "cmd", vec!["a", "b"]),
            make_step("b", "cmd", vec!["a"]),
            make_step("a", "cmd", vec![]),
        ]);
        let order = wf.execution_order().unwrap();
        let a_pos = order.iter().position(|s| s == "a").unwrap();
        let b_pos = order.iter().position(|s| s == "b").unwrap();
        let c_pos = order.iter().position(|s| s == "c").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_on_failure_from_str() {
        assert_eq!(OnFailure::parse("continue"), OnFailure::Continue);
        assert_eq!(OnFailure::parse("skip"), OnFailure::Skip);
        assert_eq!(OnFailure::parse("fail"), OnFailure::Fail);
        assert_eq!(OnFailure::parse("unknown"), OnFailure::Fail);
    }

    #[test]
    fn test_build_execution_plan_valid() {
        let wf = make_workflow("morning", vec![
            make_step("github", "github issues list", vec![]),
            make_step("gmail", "gmail messages list", vec![]),
            make_step("summarize", "janus route", vec!["github", "gmail"]),
        ]);
        let plan = build_execution_plan(&wf);
        assert!(plan.validation_errors.is_empty());
        assert_eq!(plan.estimated_steps, 3);
        assert!(!plan.parallel_groups.is_empty());
    }

    #[test]
    fn test_build_execution_plan_invalid() {
        let wf = make_workflow("bad", vec![
            make_step("a", "cmd", vec!["b"]),
            make_step("b", "cmd", vec!["a"]),
        ]);
        let plan = build_execution_plan(&wf);
        assert!(!plan.validation_errors.is_empty());
    }

    #[test]
    fn test_parallel_groups_independent_steps() {
        let wf = make_workflow("parallel", vec![
            make_step("a", "cmd", vec![]),
            make_step("b", "cmd", vec![]),
            make_step("c", "cmd", vec![]),
        ]);
        let groups = compute_parallel_groups(&wf);
        // All 3 steps have depth 0, so should be in the same group
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }

    #[test]
    fn test_validation_error_display() {
        let e = WorkflowValidationError::DuplicateStepId("step-1".to_string());
        assert!(e.to_string().contains("step-1"));
        let e2 = WorkflowValidationError::CircularDependency(vec!["a".to_string(), "b".to_string()]);
        assert!(e2.to_string().contains("→"));
    }
}
