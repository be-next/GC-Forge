# API freeze — iteration 1 (bootstrap)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

Bootstrap iteration: no public Rust API beyond crate declarations. No CLI surface beyond `gc-forge --version` (a placeholder). No YAML schema. No invariant. Doc surface is skeletal.

## Public Rust surface added

| Crate                  | Public symbol           | Notes                                   |
|------------------------|-------------------------|-----------------------------------------|
| `gc-forge-cli`         | `fn main()`             | binary entry; prints version and exits  |
| `gc-forge-scenario`    | `pub fn version() -> &'static str` | placeholder, returns crate version |
| `gc-forge-regimes`     | `pub fn version() -> &'static str` | placeholder                          |
| `gc-forge-runner`      | `pub fn version() -> &'static str` | placeholder                          |
| `gc-forge-validate`    | `pub fn version() -> &'static str` | placeholder                          |
| `gc-forge-presets`     | `pub fn version() -> &'static str` | placeholder                          |

Rationale: each crate needs at least one public symbol so doctest discovery and coverage tooling can attach. Real APIs land starting iteration 2 (`gc-forge-scenario`).

## Java harness public surface

| Class                                       | Method               | Notes                                |
|---------------------------------------------|----------------------|--------------------------------------|
| `dev.gcforge.harness.WorkloadHarness`       | `public static void main(String[])` | hello-world allocation loop |

## YAML schema changes

None.

## Invariants for Tester-unit

- `gc-forge --version` exits 0 and prints a non-empty version string.
- Each crate's `version()` returns its `Cargo.toml` version.
- The harness `main` runs to completion within 10 s without throwing, on Temurin 21.

## Doc sections to author (Doc-writer)

- `doc/process/orchestration.md` — durable copy of the implementation plan.
- `doc/user/getting-started.md` — skeleton with sections: Overview, Prerequisites, Install, First scenario, Next steps. Sections marked `_TODO iter N_` for content to come.
- `CHANGELOG.md` — `Unreleased` section initialized.

## Approval

- Coder: A1 — frozen 2026-04-25
- Reviewer: A2 — `Approved: A2 2026-04-25` (self-review acceptable for bootstrap, no prior code to review against)
