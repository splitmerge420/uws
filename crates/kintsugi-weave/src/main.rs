// crates/kintsugi-weave/src/main.rs
//
// Kintsugi Weave CLI — `kintsugi-weave`
//
// Thin orchestrator binary that runs the Kintsugi Weave gates and produces a
// GitHub PR comment markdown report.
//
// Usage:
//   kintsugi-weave run --pr-body "..." [--pr-number N] [--sha SHA]
//   kintsugi-weave format-comment --pr-body "..." [--pr-number N] [--sha SHA]
//   kintsugi-weave npfm-gate
//   kintsugi-weave provenance-validate --pr-body "..."
//   kintsugi-weave swarm-summary

use clap::{Parser, Subcommand};
use kintsugi_weave::{
    format_comment, npfm_gate, provenance_validate, run_all, swarm_commander_summary,
};

#[derive(Parser)]
#[command(
    name = "kintsugi-weave",
    about = "Kintsugi Weave — Regenerative CI engine (NPFM gate, HITL provenance, Swarm Commander)",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run all gates and print a JSON report.
    Run {
        /// Full PR body text (passed from ${{ github.event.pull_request.body }}).
        #[arg(long, default_value = "")]
        pr_body: String,

        /// PR number (optional, used for display).
        #[arg(long)]
        pr_number: Option<u64>,

        /// Commit SHA (optional, used for display).
        #[arg(long)]
        sha: Option<String>,
    },

    /// Run all gates and print a GitHub Markdown comment (for posting via gh).
    FormatComment {
        /// Full PR body text.
        #[arg(long, default_value = "")]
        pr_body: String,

        /// PR number (optional).
        #[arg(long)]
        pr_number: Option<u64>,

        /// Commit SHA (optional).
        #[arg(long)]
        sha: Option<String>,
    },

    /// Run only the NPFM gate and print its result as JSON.
    ///
    /// STUB — always returns score 0.0 until PR #7 is wired.
    NpfmGate,

    /// Run only the provenance validation gate and print its result as JSON.
    ProvenanceValidate {
        /// Full PR body text to inspect for the GoldenTrace-ID trailer.
        #[arg(long, default_value = "")]
        pr_body: String,
    },

    /// Run only the Swarm Commander summary gate and print its result as JSON.
    ///
    /// STUB — always returns PASS until swarm review integration is wired.
    SwarmSummary,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            pr_body,
            pr_number,
            sha,
        } => {
            let report = run_all(&pr_body, pr_number, sha);
            let obj = serde_json::json!({
                "pr_number": report.pr_number,
                "sha": report.sha,
                "npfm": {
                    "score": report.npfm.score,
                    "verdict": report.npfm.verdict,
                    "is_stub": report.npfm.is_stub,
                },
                "provenance": {
                    "has_trailer": report.provenance.has_trailer,
                    "trailer_value": report.provenance.trailer_value,
                    "status": report.provenance.status,
                    "is_stub": report.provenance.is_stub,
                },
                "swarm": {
                    "verdict": report.swarm.verdict,
                    "is_stub": report.swarm.is_stub,
                },
            });
            println!("{}", serde_json::to_string_pretty(&obj).unwrap());
        }

        Command::FormatComment {
            pr_body,
            pr_number,
            sha,
        } => {
            let report = run_all(&pr_body, pr_number, sha);
            print!("{}", format_comment(&report));
        }

        Command::NpfmGate => {
            let result = npfm_gate();
            let obj = serde_json::json!({
                "score": result.score,
                "verdict": result.verdict,
                "is_stub": result.is_stub,
            });
            println!("{}", serde_json::to_string_pretty(&obj).unwrap());
        }

        Command::ProvenanceValidate { pr_body } => {
            let result = provenance_validate(&pr_body);
            let obj = serde_json::json!({
                "has_trailer": result.has_trailer,
                "trailer_value": result.trailer_value,
                "status": result.status,
                "is_stub": result.is_stub,
            });
            println!("{}", serde_json::to_string_pretty(&obj).unwrap());
        }

        Command::SwarmSummary => {
            let result = swarm_commander_summary();
            let obj = serde_json::json!({
                "verdict": result.verdict,
                "is_stub": result.is_stub,
            });
            println!("{}", serde_json::to_string_pretty(&obj).unwrap());
        }
    }
}
