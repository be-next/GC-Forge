#!/usr/bin/env bash
# Build the GC-Insight learning corpus on G1 + Temurin {17, 21}.
#
# Output layout:
#   out/corpus-g1/<jdk>/<preset>-seed<N>/{gc.log, manifest.yaml}
#   out/corpus-g1/INDEX.csv  (one row per run)
#
# Each run uses the preset's native duration. Total wall-clock ~70-90 min
# on a workstation. Resumable: existing log files are skipped.

set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

GC_FORGE="${GC_FORGE:-./target/release/gc-forge}"
OUT_ROOT="${OUT_ROOT:-out/corpus-g1}"
SEEDS=(0xC0FFEE 0xBEEFCAFE)

PRESETS=(
  steady-g1-baseline
  burst-g1-30s
  humongous-g1-classic
  humongous-g1-evac-fail
  leak-g1-slow
  cache-g1-churn
  mixed-pathological-g1
  microservice-g1-stop-go
)

JDKS=(17 21)

mkdir -p "$OUT_ROOT"
INDEX="$OUT_ROOT/INDEX.csv"
# Always rebuild the index so it stays consistent with the on-disk state,
# even when the script is resumed mid-corpus.
echo "preset,jdk,seed,duration_s,log_path,manifest_path,exit_status,started_at,finished_at" > "$INDEX"

total=$(( ${#PRESETS[@]} * ${#JDKS[@]} * ${#SEEDS[@]} ))
i=0

for jdk in "${JDKS[@]}"; do
  image="gc-forge-runner:dev-jdk${jdk}"
  jdk_dir="$OUT_ROOT/jdk${jdk}"
  mkdir -p "$jdk_dir"
  for preset in "${PRESETS[@]}"; do
    for seed in "${SEEDS[@]}"; do
      i=$((i + 1))
      cell_dir="$jdk_dir/${preset}-seed${seed#0x}"
      mkdir -p "$cell_dir"
      seed_lower=$(printf '%s' "$seed" | tr 'A-FX' 'a-fx' | sed 's/^0x//')
      log="${cell_dir}/${preset}-${seed_lower}.log"
      manifest="${cell_dir}/${preset}-${seed_lower}.manifest.yaml"

      if [[ -s "$log" && -s "$manifest" ]]; then
        printf '[%2d/%d] SKIP existing %s\n' "$i" "$total" "$cell_dir"
        printf '%s,%s,%s,,%s,%s,0,resumed,resumed\n' \
          "$preset" "$jdk" "$seed" "$log" "$manifest" >> "$INDEX"
        continue
      fi

      printf '[%2d/%d] RUN  jdk%s %s seed=%s\n' "$i" "$total" "$jdk" "$preset" "$seed"
      started_epoch=$(date -u +%s)
      started_at=$(date -u +%FT%TZ)
      set +e
      "$GC_FORGE" run "presets/${preset}.yaml" \
        --out-dir "$cell_dir" \
        --image "$image" \
        --embedded-harness /opt/gc-forge/harness.jar \
        --override "spec.jvm.major=${jdk}" \
        --override "spec.seed=${seed}" \
        > "${cell_dir}/run.stdout.log" 2> "${cell_dir}/run.stderr.log"
      exit_code=$?
      set -e
      finished_epoch=$(date -u +%s)
      finished_at=$(date -u +%FT%TZ)
      duration_s=$((finished_epoch - started_epoch))

      printf '%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
        "$preset" "$jdk" "$seed" "$duration_s" "$log" "$manifest" \
        "$exit_code" "$started_at" "$finished_at" >> "$INDEX"
    done
  done
done

echo
echo "Corpus built under $OUT_ROOT"
echo "Index: $INDEX"
wc -l "$INDEX"
