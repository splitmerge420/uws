// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! `uws swarm` — multi-agent coordination commands.
//!
//! # Subcommands
//! - `uws swarm review <pr-ref>` — dispatch a PR or branch to the multi-agent
//!   review path.
//!
//! # Status
//! The swarm review backend is **not yet implemented**. The `review` subcommand
//! prints a description of what it would do and exits cleanly. Once the backend
//! (Janus v2 multi-agent router or equivalent) is wired in, replace the stub
//! body in `handle_review` with the real dispatch call.
//!
//! TODO: Wire `handle_review` to Janus v2 multi-agent router once PR #14 /
//!       the swarm backend is merged and stabilised.

use crate::error::GwsError;
use clap::{Arg, ArgMatches, Command};

/// Build the top-level `swarm` command.
pub fn build_swarm_command() -> Command {
    Command::new("swarm")
        .about("Multi-agent swarm coordination")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(build_review_command())
}

/// Build the `swarm review` subcommand.
fn build_review_command() -> Command {
    Command::new("review")
        .about("Dispatch a PR or branch to the multi-agent review path")
        .arg(
            Arg::new("ref")
                .help("PR number (e.g. 42), branch name, or full GitHub ref to review")
                .value_name("PR_REF")
                .required(true),
        )
        .arg(
            Arg::new("repo")
                .long("repo")
                .short('r')
                .help("Repository in owner/name format (default: current repo from git remote)")
                .value_name("OWNER/REPO"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .help("Output format: json (default), table, yaml")
                .value_name("FORMAT")
                .default_value("json"),
        )
        .after_help(
            "\
EXAMPLES:
  uws swarm review 42
  uws swarm review my-feature-branch --repo atlaslattice/uws
  uws swarm review 42 --format table

NOTES:
  This command dispatches to the multi-agent review backend (Janus v2 router).
  The backend is not yet implemented; a stub response is returned instead.
  See: https://github.com/atlaslattice/uws/issues (track swarm review backend)
",
        )
}

/// Handle the `swarm` command tree.
///
/// Returns `Ok(())` if the command was handled, or a `GwsError` on failure.
pub async fn handle_swarm_command(args: &[String]) -> Result<(), GwsError> {
    let cmd = build_swarm_command();
    let matches = cmd
        .try_get_matches_from(std::iter::once("swarm".to_string()).chain(args.iter().cloned()))
        .map_err(|e| {
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                print!("{e}");
                std::process::exit(0);
            }
            GwsError::Validation(e.to_string())
        })?;

    match matches.subcommand() {
        Some(("review", sub)) => handle_review(sub).await,
        _ => Err(GwsError::Validation(
            "Unknown swarm subcommand. Run `uws swarm --help` for usage.".to_string(),
        )),
    }
}

/// Handle `uws swarm review <pr-ref>`.
///
/// # TODO
/// Replace the stub body with a call to the Janus v2 multi-agent router once
/// the backend is available. The router should:
/// 1. Resolve `pr_ref` to a concrete diff set via the GitHub API.
/// 2. Fan out to multiple specialist review agents (security, performance,
///    style, test-coverage, docs).
/// 3. Aggregate results and emit a structured review report (JSON or table).
/// 4. Append a `GoldenTrace` provenance trailer to the PR description.
async fn handle_review(matches: &ArgMatches) -> Result<(), GwsError> {
    let pr_ref = matches
        .get_one::<String>("ref")
        .expect("ref is required by clap");
    let repo = matches
        .get_one::<String>("repo")
        .map(|s| s.as_str())
        .unwrap_or("<inferred from git remote>");
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("json");

    // ── Stub response ────────────────────────────────────────────────────────
    // TODO: replace with actual swarm dispatch once Janus v2 router is wired.
    // unimplemented!("swarm review backend not yet implemented")

    let stub_response = serde_json::json!({
        "status": "stub",
        "message": "Swarm review backend is not yet implemented. \
                    This command will dispatch the PR to the multi-agent review \
                    path once the Janus v2 router is wired in.",
        "pr_ref": pr_ref,
        "repo": repo,
        "would_do": [
            "Resolve PR diff via GitHub API",
            "Dispatch to security review agent",
            "Dispatch to performance review agent",
            "Dispatch to style / lint review agent",
            "Dispatch to test-coverage review agent",
            "Dispatch to documentation review agent",
            "Aggregate and rank findings",
            "Post structured review comment to PR",
            "Append GoldenTrace provenance trailer to PR description"
        ]
    });

    match format {
        "table" => {
            println!("Status : {}", stub_response["status"].as_str().unwrap_or(""));
            println!("PR ref : {pr_ref}");
            println!("Repo   : {repo}");
            println!("Note   : {}", stub_response["message"].as_str().unwrap_or(""));
        }
        _ => {
            println!("{}", serde_json::to_string_pretty(&stub_response).unwrap());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_swarm_command_has_review() {
        let cmd = build_swarm_command();
        let subs: Vec<_> = cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(subs.contains(&"review"), "swarm must have a 'review' subcommand");
    }

    #[test]
    fn test_review_command_requires_ref() {
        let cmd = build_swarm_command();
        // Missing required arg → should fail
        let result = cmd.try_get_matches_from(vec!["swarm", "review"]);
        assert!(result.is_err(), "review without <PR_REF> must fail");
    }

    #[test]
    fn test_review_command_accepts_ref() {
        let cmd = build_swarm_command();
        let m = cmd
            .try_get_matches_from(vec!["swarm", "review", "42"])
            .expect("review with PR ref should parse OK");
        let (sub, sub_m) = m.subcommand().unwrap();
        assert_eq!(sub, "review");
        assert_eq!(
            sub_m.get_one::<String>("ref").map(|s| s.as_str()),
            Some("42")
        );
    }

    #[test]
    fn test_review_command_accepts_repo_flag() {
        let cmd = build_swarm_command();
        let m = cmd
            .try_get_matches_from(vec!["swarm", "review", "42", "--repo", "atlaslattice/uws"])
            .expect("should parse with --repo");
        let (_, sub_m) = m.subcommand().unwrap();
        assert_eq!(
            sub_m.get_one::<String>("repo").map(|s| s.as_str()),
            Some("atlaslattice/uws")
        );
    }

    #[tokio::test]
    async fn test_handle_swarm_review_stub() {
        // Should not panic; returns Ok with stub JSON on stdout.
        let args: Vec<String> = vec!["review".to_string(), "99".to_string()];
        let result = handle_swarm_command(&args).await;
        assert!(result.is_ok(), "stub handler must not fail: {result:?}");
    }
}
