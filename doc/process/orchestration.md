# Implementation orchestration

This document describes how GC-Forge is built. It is the durable
in-tree counterpart of the implementation plan and applies to every
iteration up to and including the MVP 0.1.0 release.

## Goals

- Ship the MVP `0.1.0` (release candidate at the end of Phase 3 of `doc/specs/ROADMAP.md`).
- Maximise test coverage with a progressive floor (70 % → 85 %).
- Keep user-facing documentation in sync with the code, iteration by iteration.
- Run the implementation as a multi-agent loop with rotating roles, with explicit gates and stop criteria.

The plan does not re-decide any technical choice fixed in `doc/specs/`. It defines the *execution process*.

## Iteration

An iteration is a single unit of work, scoped to one feature of the roadmap backbone (typically 3 to 5 days of equivalent work). Each iteration lives on a branch named `iter/NN-short-title` and is fast-forward merged onto `main` once the Definition-of-Done gate passes. The full backbone is listed in the implementation plan; the next iteration to start is always the lowest-numbered open one.

## Roles

Six roles cover the work. They rotate every iteration, with the constraint that no agent holds the same role two iterations in a row. The Teamlead leaving an iteration proposes the rotation for the next one and may veto a rotation when a critical skill mismatch is identified.

| Role | Responsibilities |
|------|------------------|
| Coder       | Implements the iteration's feature. Publishes `API-FREEZE.md`. |
| Reviewer    | Reviews architecture, security, spec alignment, performance. Signs `API-FREEZE.md`. Can block the merge. |
| Tester-unit | Writes unit tests in Rust (`cargo test`) and Java (JUnit 5). Drives coverage. |
| Tester-func | Runs integration tests that launch real JVMs in Docker, on impacted presets and the smoke selftest. Maintains `BUGS.md`. |
| Doc-writer  | Maintains `doc/user/`, `CHANGELOG.md`, `README.md`, and from iteration 13 onwards `doc/concepts/traceability.md`. |
| Teamlead    | Arbitrates edge cases, manages role rotation, writes the `ITERATION-LOG.md` entry, decides the merge after the DoD gate is green. |

## Iteration flow

```
1. Teamlead briefs the team.
   Reads ITERATION-LOG.md, BUGS.md, picks the next backbone feature, assigns roles.
2. Coder produces an implementation draft and writes API-FREEZE.md.
3. Reviewer signs API-FREEZE.md.                                ◀── Gate 1
4. Coder finalises ║ Tester-unit writes tests ║ Doc-writer writes doc.   (parallel)
5. Coder commits. Reviewer reads the diff.                      ◀── Gate 2
6. Tester-func runs integration tests on impacted presets + smoke selftest.
7. DoD gate runs.                                               ◀── Gate 3
8. Teamlead merges fast-forward and writes the iteration's ledger entry.
```

### Gate 1 — `API-FREEZE.md`

Before parallel work starts, the Coder publishes a short document at the repository root. It captures the public Rust signatures added or modified, any YAML schema change, the invariants the Tester-unit must check, and the documentation sections to author. The Reviewer signs the file (`Approved: <reviewer-id> <date>`); without that signature, the Tester-unit and Doc-writer wait. The file is overwritten at every iteration.

### Gate 2 — code review

After the Coder pushes the final commit, the Reviewer reads the full diff. A red flag (architecture, security, spec drift) blocks the merge regardless of the DoD gate.

### Gate 3 — Definition-of-Done

`scripts/dod-gate.sh` is the machine-verifiable contract. It must return zero. The full check list is documented at the top of the script; floors progress with the phase (coverage 0 % → 70 % → 80 % → 85 %). The Teamlead has no override: a failing check that is a legitimate environment issue is documented as an exception in `ITERATION-LOG.md` and addressed in the next iteration — never silently skipped.

## Bug management

`BUGS.md` is the single source of truth for bugs found during implementation, owned by the Tester-func role. Severity dictates flow:

- `critical` or `high` open ⇒ DoD gate is red. The current iteration must fix the bug before merge or escalate.
- `medium` and `low` are queued and prioritised by the Teamlead.
- Hard cap: more than 15 `medium` bugs open, or a `medium` left untouched for more than three iterations, triggers an automatic escalation. This is treated as a stagnation signal.

## Stop criteria

The loop stops on one of four conditions:

1. **Completion.** Iteration 17 merged, `gc-forge selftest` green on the shipped presets (currently 21), and the artefacts of release `0.1.0` are ready (the `v0.1.0` tag itself requires human approval, see "Boundaries" below).
2. **Blocking.** The Teamlead raises a flag for an unresolvable architectural ambiguity, an external dependency, or any action requiring a human decision. The loop pauses, the context is recorded in `ITERATION-LOG.md`, and a clear escalation message is emitted.
3. **Stagnation.** Any of: (a) two consecutive iterations without a merge and coverage stable to within ±0.5 pt; (b) one iteration without a merge while at least one `high` bug is open; (c) the BUGS.md hard cap is hit; (d) an iteration timeout is reached (more than five calendar days, or more than eight loop ticks without a merge).
4. **Manual interruption.** The user interrupts; the loop finishes the current step at a coherent point and stops.

## Boundaries

Some actions are reversible only with effort or affect parties beyond the local environment. They are out of scope for the autonomous loop and require a human decision:

- Pushing branches or tags to a remote.
- Publishing crates to crates.io.
- Pushing Docker images to GHCR or any registry.
- Creating release tags (`v0.1.0`, etc.).
- Modifying `LICENSE` or shared cross-project artefacts (notably any change to `gc-core` that affects GC-Insight).

When iteration 17 reaches the point where the release tag is the only remaining step, the loop stops and escalates to the human operator with the necessary context.

## Pilot artefacts

| File | Owner | Purpose |
|------|-------|---------|
| `BUGS.md` | Tester-func | Live bug backlog. |
| `ITERATION-LOG.md` | Teamlead | One ledger entry per iteration: feature, roles, metrics, decisions, post-mortem. |
| `API-FREEZE.md` | Coder + Reviewer | Gate 1; overwritten each iteration. |
| `CHANGELOG.md` | Doc-writer | Keep-a-Changelog. |
| `doc/user/*.md` | Doc-writer | User documentation, kept in sync with the code. |
| `doc/concepts/traceability.md` | Doc-writer (from iter 13) | Matrix of phenomenon × preset × insight-capability. |
| `scripts/dod-gate.sh` | Coder (iter 1) | Gate 3. |
| `.github/workflows/ci.yml` | Coder (iter 1) | Continuous integration. |

## Release procedure

The release of `v0.1.0` (and every subsequent release) is the
intersection of CI automation and a small set of human-gated steps.
The CI definition lives in `.github/workflows/release.yml`. The
human steps below are deliberately **not** automated — they are the
boundary between the autonomous loop and the operator.

### Pre-release checklist (before tagging)

1. `main` is at the version that should be released (workspace
   `Cargo.toml` and `Cargo.lock` agree).
2. `CHANGELOG.md` has a dated section for that version (no
   `[Unreleased]` entries below it).
3. `scripts/dod-gate.sh` exits 0.
4. `gc-forge selftest` is green on every shipped preset, ideally
   under faithful (non-truncated) durations.
5. Every iteration's bilan is in `ITERATION-LOG.md`; no `high`/
   `critical` bug is open in `BUGS.md`.

### Human-gated steps (in order)

These four actions are external and non-reversible. The autonomous
loop **must not** perform them; it stops at the merge of the release
iteration with everything in place locally.

1. **Push `main` to origin.**
   `git push origin main` — propagates every accumulated commit since
   the previous release. Run this first so the tag in step 2 lands on
   a public commit.
2. **Create and push the annotated tag.**
   ```sh
   git tag -a vX.Y.Z -m "GC-Forge X.Y.Z"
   git push origin vX.Y.Z
   ```
   The tag push triggers `.github/workflows/release.yml`. The
   `build-cli` matrix runs immediately. The `publish-crates` and
   `publish-docker` jobs *halt waiting for approval* on the
   `release` GitHub environment.
3. **Approve the `release` environment runs** in the GitHub UI (one
   approval per gated job). `publish-crates` walks the six crates in
   topological order; if any crate fails, fix it on a follow-up patch
   release rather than retrying the same tag. `publish-docker`
   pushes `ghcr.io/<org>/gc-forge:X.Y.Z` and `:latest`.
4. **Promote the GitHub Release from draft to public.** The
   `build-cli` job uploads the four `.tar.gz` archives to a draft
   release; review the auto-generated notes against `CHANGELOG.md`,
   edit, then publish.

### Required GitHub configuration (one-time)

- **`release` environment** with:
    - required reviewers: at least one repo admin;
    - secret `CARGO_REGISTRY_TOKEN`: a crates.io token scoped to
      publishing the six `gc-forge-*` crates.
- **`packages: write` permission** on the default `GITHUB_TOKEN` is
  the only thing `publish-docker` needs (set in the workflow).
- **Branch protection on `main`** with a passing `ci` workflow
  required.

### Rollback procedure

If a `publish-crates` job partially succeeds (some crates uploaded,
others not) **do not** force a retry of the same tag — crates.io is
append-only. Instead:

1. Yank the partially-published crates with `cargo yank --vers X.Y.Z
   <crate>`. (Yanking is reversible in the metadata sense; the file
   stays accessible to existing lockfiles.)
2. Cut a `X.Y.Z+1` patch tag with the fix and walk the procedure
   again from step 1.

For `publish-docker`, overwriting the `:latest` tag on retry is
acceptable; the multi-architecture manifest then points at the new
build.

## References

- Implementation plan: `/Users/jerome/.claude/plans/ok-partout-structured-feather.md` (working copy; this file is the durable, in-tree counterpart).
- Specifications: `doc/specs/SPEC-FUNCTIONAL.md`, `doc/specs/SPEC-TECHNICAL.md`, `doc/specs/ROADMAP.md`, `doc/specs/RISKS.md`, `doc/specs/BACKLOG.md`.
- Release workflow: `.github/workflows/release.yml`.
