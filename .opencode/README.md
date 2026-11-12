# Project agents

OpenCode discovers every `*.md` file here as an agent; the Markdown body is the
agent's system prompt. See <https://opencode.ai/docs/agents>.

| Agent | Mode | Purpose |
| --- | --- | --- |
| `rust-engineer` | all | Implements and reviews Rust with measured optimization, 100% coverage and exhaustive edge-case tests. |

Usage: select `rust-engineer` as the primary agent, or ask the current agent to
launch it as a subagent for an implementation task.

## House rules encoded in `rust-engineer`

- Correctness first; a false positive is worse than a false negative.
- Optimization only with a before/after benchmark; budgets live in
  `.planning/PLAN.md` §2.4.
- 100% line/region/function coverage, enforced with `cargo llvm-cov`.
- Every boundary, malformed, adversarial and valid-but-unusual input gets a test;
  property tests, fuzzing, mutation testing, differential and snapshot tests are
  part of the definition of done.
- Pure Rust only: no C toolchain, no `f64`, no runtime regex, no DOM.
- Every rule cites its official SdI code and the source section.

Add new agents as `<name>.md` in this folder. Keep each agent's rules in its own
body; project-wide facts belong in `.planning/PLAN.md`.
