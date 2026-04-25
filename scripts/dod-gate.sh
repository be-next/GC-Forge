#!/usr/bin/env bash
# DoD gate — Definition-of-Done checks that must all pass for an iteration to merge.
#
# The set of checks is intentionally progressive: it expands as the codebase
# grows. Each iteration's ITERATION-LOG.md notes what is enforced and what is
# deferred.
#
# Usage: scripts/dod-gate.sh [--phase=N]
#   --phase=N  Override the auto-detected phase. Mostly for CI.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." &>/dev/null && pwd)"
cd "${REPO_ROOT}"

PHASE="${PHASE:-1}"
for arg in "$@"; do
    case "${arg}" in
        --phase=*) PHASE="${arg#--phase=}" ;;
        *) echo "Unknown arg: ${arg}" >&2; exit 2 ;;
    esac
done

# Coverage floor by phase, per the implementation plan.
case "${PHASE}" in
    0|1) COVERAGE_FLOOR=0   ;;  # bootstrap & early phase 1: no floor yet
    2)   COVERAGE_FLOOR=70  ;;
    3)   COVERAGE_FLOOR=85  ;;
    *)   COVERAGE_FLOOR=85  ;;
esac

step() { printf '\n\033[36m▶ %s\033[0m\n' "$*"; }
ok()   { printf '\033[32m✓ %s\033[0m\n' "$*"; }
skip() { printf '\033[33m∼ %s (skipped this phase)\033[0m\n' "$*"; }
fail() { printf '\033[31m✗ %s\033[0m\n' "$*"; exit 1; }

# Each check below uses an explicit if/then/else so that `set -e` cannot
# mask a failure that hides behind `&&` short-circuiting.

# 1. Rust formatting
step "cargo fmt --check"
if cargo fmt --all -- --check; then
    ok "fmt"
else
    fail "rustfmt found differences (run \`cargo fmt --all\`)"
fi

# 2. Rust clippy
step "cargo clippy"
if cargo clippy --workspace --all-targets -- -D warnings; then
    ok "clippy"
else
    fail "clippy found issues"
fi

# 3. Rust tests
step "cargo test"
if cargo test --workspace --quiet; then
    ok "cargo test"
else
    fail "rust tests failed"
fi

# 4. cargo-deny (licenses + advisories + bans + sources)
step "cargo deny check"
if command -v cargo-deny >/dev/null 2>&1; then
    if cargo deny check; then
        ok "deny"
    else
        fail "cargo deny found issues"
    fi
else
    skip "cargo-deny not installed (install with: cargo install --locked cargo-deny)"
fi

# 5. Rust coverage
step "rust coverage (floor=${COVERAGE_FLOOR}%)"
if [[ "${COVERAGE_FLOOR}" -eq 0 ]]; then
    skip "coverage floor 0 in phase ${PHASE}"
elif command -v cargo-llvm-cov >/dev/null 2>&1; then
    if cargo llvm-cov --workspace --fail-under-lines "${COVERAGE_FLOOR}"; then
        ok "coverage ≥ ${COVERAGE_FLOOR}%"
    else
        fail "rust coverage below ${COVERAGE_FLOOR}%"
    fi
else
    skip "cargo-llvm-cov not installed (install with: cargo install --locked cargo-llvm-cov)"
fi

# 6. Java harness verify
step "mvn verify (workload-harness)"
if command -v mvn >/dev/null 2>&1; then
    if mvn -f workload-harness/pom.xml -q verify; then
        ok "mvn verify"
    else
        fail "mvn verify failed"
    fi
else
    skip "Maven not installed"
fi

# 7. selftest (lands iteration 15)
step "gc-forge selftest"
if [[ "${PHASE}" -ge 3 ]] && command -v gc-forge >/dev/null 2>&1; then
    gc-forge selftest && ok "selftest"
else
    skip "selftest not yet wired (lands iter 15) or gc-forge not on PATH"
fi

# 8. variance-check (lands iteration 15)
step "gc-forge variance-check (touched presets)"
if [[ "${PHASE}" -ge 3 ]] && command -v gc-forge >/dev/null 2>&1; then
    skip "variance-check on touched presets is wired by Teamlead per iteration"
else
    skip "variance-check not yet available"
fi

# 9. No `high`/`critical` bugs open in BUGS.md
step "BUGS.md — no high/critical open"
blocking_bug() {
    awk '
        /^```/             { fence = !fence; next }
        fence              { next }
        /^## BUG-/ {
            if (sev ~ /(high|critical)/ && status == "open") { found=1 }
            sev=""; status=""
            next
        }
        /^- Severity:/ { sev=$3 }
        /^- Status:/   { status=$3 }
        END {
            if (sev ~ /(high|critical)/ && status == "open") { found=1 }
            exit found ? 1 : 0
        }
    ' BUGS.md
}
if blocking_bug; then
    ok "no blocking bug open"
else
    fail "at least one high/critical bug is open in BUGS.md"
fi

printf '\n\033[32m✓ DoD gate green (phase %s)\033[0m\n' "${PHASE}"
