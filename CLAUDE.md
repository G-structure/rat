# CLAUDE.md — RAT (Rust-only) Assistant Configuration
This repository is Rust-only and expects small, test-first changes with strong anti-reward-hacking guardrails. Follow this file verbatim. [Anthropic - Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)

## 0) Scope and identity
- Language: **Rust only**. Do not introduce JS or TS. Keep the codebase single-language. [Anthropic - Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)
- Product: **RAT - Rust Agent Terminal**, ACP client with TUI via `ratatui` and async via `tokio`. [Agent Client Protocol - GitHub](https://github.com/zed-industries/agent-client-protocol)
- Mode of work: headless, test-driven development with minimal diffs and explicit verification. [nexte.st](https://nexte.st/)

## 1) Working agreement for Claude
1. **Plan → test → code → verify → summarize** for every change. Propose tests first, then ship the smallest diff that turns red to green. [Anthropic - Prompt Best Practices](https://docs.anthropic.com/en/docs/build-with-claude/prompt-engineering/claude-4-best-practices)
2. Keep patches at most **150 LOC** per commit. No cross-cutting refactors without explicit instruction. [Anthropic - Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)
3. If blocked or uncertain, output an **ASK-LIST** with precise questions and stop. [Anthropic - Prompt Overview](https://docs.anthropic.com/en/docs/build-with-claude/prompt-engineering/overview)
4. Never block the TUI loop. Long work must be async. Prefer non-blocking, structured logging. [Anthropic - Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)

## 2) Commands (headless)
- **Build**: `cargo build --locked --all-features` [cargo build](https://doc.rust-lang.org/cargo/commands/cargo-build.html)
- **Format**: `cargo fmt --all --check` [rustfmt](https://github.com/rust-lang/rustfmt)
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings` [Clippy](https://doc.rust-lang.org/clippy/)
- **Test - fast**: `cargo nextest run --all --no-capture` [nexte.st](https://nexte.st/)
- **Test - fallback**: `cargo test --all` [nexte.st](https://nexte.st/)
- **Snapshots (TUI and text)**: `cargo insta test --review` [insta](https://insta.rs/docs/)
- **Mutation tests**: `cargo mutants --test-tool=nextest` [cargo-mutants](https://mutants.rs/)
- **Run RAT**: `cargo run -- --agent claude-code -v` and adjust flags as needed. [Zed Docs - External Agents](https://zed.dev/docs/ai/external-agents)

## 3) Repository test contract
Prefer **integration tests** and typed oracles over brittle string checks. Test stack guidance follows. [Anthropic - Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)
- **CLI flows**: `assert_cmd` with `predicates` to run `rat` and assert exit status and semantics. [assert_cmd](https://docs.rs/assert_cmd)
- **Snapshots**: `insta` for pretty or structured output and TUI frames with reviewed snapshots. [insta](https://insta.rs/docs/)
- **TUI recipe**: follow Ratatui test snapshots guidance. [Ratatui - Snapshots recipe](https://ratatui.rs/recipes/testing/snapshots/)
- **Properties**: `proptest` for invariants like routing, parsing, and diffs. [proptest book](https://altsysrq.github.io/proptest-book/)
- **Compile-fail**: `trybuild` for diagnostics, trait bounds, and macro errors. [trybuild](https://docs.rs/trybuild)
- **Speed and reliability**: prefer **nextest** by default. [nexte.st](https://nexte.st/)
- **Anti-cheat**: run **cargo-mutants**, target ≥ 80% killed, justify survivors. [cargo-mutants](https://mutants.rs/)

## 4) Anti-reward-hacking guardrails
1. **Fail-first**: add or adjust tests that fail before code changes, then make them pass. [Anthropic - Prompt Overview](https://docs.anthropic.com/en/docs/build-with-claude/prompt-engineering/overview)
2. **Metamorphic checks**: add at least two relations for complex behavior, for example idempotence of a formatting pass or monotonicity with added context. [DeepMind - Specification Gaming](https://deepmind.google/discover/blog/specification-gaming-the-flip-side-of-ai-ingenuity/)
3. **Negative tests**: include malformed inputs and timeouts where applicable. [Anthropic - Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)
4. **Mutation gate**: CI fails if mutants survive without a written rationale. [cargo-mutants](https://mutants.rs/)
5. **No vacuous oracles**: avoid plain string assertions on logs. Prefer exit codes, JSON structures, or typed results. [assert_cmd](https://docs.rs/assert_cmd)

## 5) Coding standards (Rust)
- **Style**: `rustfmt --check` in CI. [rustfmt](https://github.com/rust-lang/rustfmt)
- **Lints**: `clippy -D warnings`. Allow exceptions only with `#[allow(...)]` and a reason. [Clippy](https://doc.rust-lang.org/clippy/)
- **Docs**: add doc comments for public items. Consider `missing_docs` for core crates. [Clippy](https://doc.rust-lang.org/clippy/)
- **Async tests**: use `#[tokio::test]` with timeouts, run under nextest, and avoid flakiness. [Tokio #[tokio::test]](https://docs.rs/tokio/latest/tokio/attr.test.html)
- **Logging**: prefer `tracing` with non-blocking writers. No sync file I/O on TUI paths. [tracing](https://docs.rs/tracing)

## 6) File boundaries and edit policy
- **Allowed**: `src/**`, `tests/**`, `examples/**`, `benches/**`, and CI config. [Anthropic - Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)
- **Avoid**: cross-cutting renames, module moves, or theme overhauls unless requested. [Anthropic - Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)
- **ACP-facing code**: follow protocol expectations and keep session and auth handling non-blocking. [Agent Client Protocol - GitHub](https://github.com/zed-industries/agent-client-protocol)

## 7) Execution template (Claude must fill and follow)

#+BEGIN_SRC text
# refs: nexte.st / insta / cargo-mutants / assert_cmd
<plan>
- Task: one sentence
- Files to touch:
- Risks:
</plan>

<tests>
- New or changed tests (assert_cmd / insta / proptest / trybuild)
- Why these oracles are robust (include metamorphic relations if applicable)
</tests>

<patch>
- Minimal diff; no unrelated changes
</patch>

<verify>
- Run: cargo nextest run --all --no-capture
- Run: cargo insta test --review
- Run: cargo mutants --test-tool=nextest
- Evidence: failures → green diff, mutants killed ≥ 80%
</verify>

<summary>
- What changed, why, and user-visible behavior
</summary>
#+END_SRC
[nexte.st](https://nexte.st/)

## 8) Test patterns (ready to copy)

**CLI - assert_cmd** [assert_cmd](https://docs.rs/assert_cmd)
#+BEGIN_SRC rust
// refs: assert_cmd / predicates
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn rat_shows_help() {
    let mut cmd = Command::cargo_bin("rat").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("RAT"));
}
#+END_SRC

**Snapshot of formatted output - insta** [insta](https://insta.rs/docs/)
#+BEGIN_SRC rust
// refs: insta
use insta::assert_snapshot;

#[test]
fn pretty_status_line() {
    let rendered = format!("status: {}", "Connected");
    assert_snapshot!(rendered);
}
#+END_SRC

**Property-based - proptest** [proptest book](https://altsysrq.github.io/proptest-book/)
#+BEGIN_SRC rust
// refs: proptest
use proptest::prelude::*;

proptest! {
    #[test]
    fn session_id_roundtrips(s in "\\PC{1,40}") {
        // Replace with a real round-trip invariant
        prop_assert!(s.len() <= 40);
    }
}
#+END_SRC

**Compile-fail - trybuild** [trybuild](https://docs.rs/trybuild)
#+BEGIN_SRC rust
// refs: trybuild
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
#+END_SRC

## 9) CI policy
- Pipeline: cache build, then `fmt` → `clippy` → `nextest` → `insta` → `mutants`. [nexte.st](https://nexte.st/)
- Gates: warnings fail CI, snapshots require review, mutation threshold enforced. [insta](https://insta.rs/docs/), [cargo-mutants](https://mutants.rs/)

## 10) One-page guardrails recap
- Fail-first tests, no vacuous oracles, and explicit oracles. [DeepMind - Specification Gaming](https://deepmind.google/discover/blog/specification-gaming-the-flip-side-of-ai-ingenuity/)
- Prefer nextest and add insta, proptest, and trybuild where it raises coverage and confidence. [nexte.st](https://nexte.st/)
- Run cargo-mutants and block on weak tests. [cargo-mutants](https://mutants.rs/)
- Rust only, responsive UI, and no sync I/O on the TUI path. [Ratatui - Snapshots recipe](https://ratatui.rs/recipes/testing/snapshots/)
