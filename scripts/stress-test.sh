#!/usr/bin/env bash
# stress-test.sh — Automate contract stress test execution and report results
# Usage: ./scripts/stress-test.sh [--release]
set -euo pipefail

RELEASE_FLAG=""
if [[ "${1:-}" == "--release" ]]; then
  RELEASE_FLAG="--release"
fi

echo "=== StellarEscrow Contract Stress Tests ==="
echo "Running: cargo test --test stress -- --nocapture $RELEASE_FLAG"
echo ""

cd "$(dirname "$0")/.."

START=$(date +%s%N)

cargo test --test stress -- --nocapture $RELEASE_FLAG 2>&1 | tee /tmp/stress_output.txt

END=$(date +%s%N)
ELAPSED=$(( (END - START) / 1000000 ))

echo ""
echo "=== Summary ==="
echo "Total wall time: ${ELAPSED}ms"
echo ""

# Extract benchmark lines for a quick report
echo "--- Benchmark Results ---"
grep -E '^\[bench\]|\[monitor\]' /tmp/stress_output.txt || true

echo ""
PASS=$(grep -c "^test .* ok$" /tmp/stress_output.txt || true)
FAIL=$(grep -c "^test .* FAILED$" /tmp/stress_output.txt || true)
echo "Passed: $PASS | Failed: $FAIL"

if [[ "$FAIL" -gt 0 ]]; then
  echo "STRESS TESTS FAILED" >&2
  exit 1
fi

echo "All stress tests passed."
