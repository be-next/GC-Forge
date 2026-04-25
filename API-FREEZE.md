# API freeze — iteration 17 (release-pipeline)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

Final iteration before tagging `v0.1.0`. **No application-level surface
changes**: this iteration is exclusively about release plumbing.

## Public Rust surface added

None. No crate adds or removes a public item. No CLI subcommand is
added. The wire formats (`gc-forge/scenario.v1`,
`gc-forge/run-manifest.v1`, `gc-forge/matrix.v1`) are frozen.

## Workspace version bump

`Cargo.toml` (workspace) lifts the version pin:

| File | Before | After |
|------|--------|-------|
| `Cargo.toml` `[workspace.package].version` | `0.0.1-dev` | `0.1.0` |
| `Cargo.toml` `[workspace.dependencies].gc-forge-*.version` | `0.0.1-dev` | `0.1.0` |

All six crates inherit `version.workspace = true`, so a single edit in
the workspace propagates everywhere. `Cargo.lock` is regenerated.

## CI artefacts added

- `.github/workflows/release.yml` — matrix release workflow, triggered
  on tags matching `v*.*.*` and on `workflow_dispatch`. Stages:
    1. **build-cli** matrix: linux-x86_64-gnu, linux-aarch64-gnu,
       macos-x86_64, macos-aarch64. Each builds `gc-forge` (release
       profile), strips, archives `gc-forge-${target}.tar.gz`,
       uploads as workflow artefact, and (on tag) attaches to the
       GitHub Release.
    2. **publish-crates** (sequential, depends on build-cli): runs
       `cargo publish` for the six crates in topological order
       (`scenario` → `regimes` → `runner` → `validate` → `presets`
       → `cli`). **Gated** on the `release` GitHub environment so a
       human approves before any crate goes out. Reads
       `secrets.CARGO_REGISTRY_TOKEN`.
    3. **publish-docker**: `docker buildx` multi-arch build of
       `Dockerfile`, tagged `ghcr.io/<org>/gc-forge:<version>` and
       `ghcr.io/<org>/gc-forge:latest`. **Gated** on the `release`
       environment. Reads `secrets.GITHUB_TOKEN` (default).

  Both publish stages skip cleanly when triggered manually
  without an explicit version (so the workflow is testable on a PR
  without secrets).

## CHANGELOG transition

`CHANGELOG.md`:
- The current `[Unreleased]` heading becomes `[0.1.0] — 2026-04-25`.
- A fresh, empty `[Unreleased]` heading is added on top.
- A new "Released" link table is added at the bottom per the
  Keep-a-Changelog convention.

## Documentation added

- `doc/process/orchestration.md` — gains a final section
  "Release procedure" describing the steps from "iter 17 merged"
  to "tag pushed", which actions are human-gated, and the
  rollback procedure.

## Files outside scope (not touched)

- `crates/**/src/`: no source change.
- `workload-harness/src/`: no source change.
- `presets/`: no preset change.
- `schemas/`: regenerated only if `cargo run -p gc-forge-cli --bin
  gen-schema` produces a diff (it should not, since no model field
  changed).
- `BUGS.md`: no new bug expected from this iteration.

## Tester-unit checklist

- `cargo build --release --workspace` still succeeds at version
  `0.1.0`.
- `cargo package --workspace --no-verify` exits clean for every
  crate (sanity check on metadata).
- `cargo test --workspace --locked` stays at 191 tests passing.
- `mvn -f workload-harness/pom.xml verify` stays at 61 tests passing.

## Tester-func checklist

- `gc-forge --version` prints `0.1.0`.
- `make build && make demo` still produces a GC log.
- `gc-forge selftest` (with truncated `--per-preset-duration`) stays
  green on all 14 presets.

## Doc-writer checklist

- `CHANGELOG.md`: cut the 0.1.0 release notes from the accumulated
  Unreleased entries, with a chronological summary at the top.
- `README.md`: status banner moves from "Pre-release of 0.1.0" to
  "0.1.0 — first MVP release".
- `doc/process/orchestration.md`: append the release procedure.

## Out of scope for iter 17 (escalation required)

The following actions are deliberately NOT performed by the loop, per
the autonomy boundary documented in `feedback_autonomy`:

1. `git push origin main` — the 25 (now 26) commits ahead of origin
   stay local.
2. `git tag v0.1.0` — the tag would auto-trigger crates.io and GHCR
   publishes via the `release.yml` workflow; both are external,
   non-reversible actions.
3. `cargo publish` — six crates would land on crates.io; the names
   `gc-forge-*` get reserved permanently on first publish.
4. `docker push ghcr.io/...` — would publish a multi-arch image
   from this machine without going through CI.

These four actions are the operator's responsibility. The loop stops
at the merge of iter 17 with everything in place locally and an
explicit escalation note in `ITERATION-LOG.md`.

---

Approved: A6 (Reviewer) — 2026-04-25.
