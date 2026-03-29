#!/usr/bin/env bash
# infra/cost/cost-report.sh
# Generate a monthly cost report using AWS Cost Explorer CLI.
# Requires: aws CLI with ce:GetCostAndUsage permission
#
# Usage:
#   ./cost-report.sh [--month YYYY-MM] [--output json|text]

set -euo pipefail

MONTH="${1:-$(date -u +%Y-%m)}"
OUTPUT_FORMAT="${2:-text}"
REPORT_DIR="$(dirname "$0")/reports"
mkdir -p "$REPORT_DIR"

START="${MONTH}-01"
# Last day of month
END=$(date -d "${START} +1 month" +%Y-%m-01 2>/dev/null || \
      python3 -c "import datetime; d=datetime.date(${MONTH:0:4},${MONTH:5:2},1); print((d.replace(day=28)+datetime.timedelta(days=4)).replace(day=1))")

REPORT_FILE="${REPORT_DIR}/cost-report-${MONTH}.json"

echo "[cost-report] Generating report for ${MONTH} (${START} → ${END})"

# ── Total cost by service ─────────────────────────────────────────────────────
aws ce get-cost-and-usage \
  --time-period "Start=${START},End=${END}" \
  --granularity MONTHLY \
  --metrics "BlendedCost" "UsageQuantity" \
  --group-by "Type=DIMENSION,Key=SERVICE" \
  --output json > "$REPORT_FILE"

# ── Parse and display ─────────────────────────────────────────────────────────
echo ""
echo "══════════════════════════════════════════════════"
echo "  StellarEscrow Cost Report — ${MONTH}"
echo "══════════════════════════════════════════════════"

python3 - <<EOF
import json, sys

with open("${REPORT_FILE}") as f:
    data = json.load(f)

results = data.get("ResultsByTime", [{}])[0]
groups = results.get("Groups", [])
total = 0.0

rows = []
for g in groups:
    service = g["Keys"][0]
    cost = float(g["Metrics"]["BlendedCost"]["Amount"])
    if cost > 0.01:
        rows.append((service, cost))
        total += cost

rows.sort(key=lambda x: -x[1])
for service, cost in rows:
    bar = "█" * int(cost / total * 30)
    print(f"  {service[:40]:<40} \${cost:>8.2f}  {bar}")

print(f"\n  {'TOTAL':<40} \${total:>8.2f}")
print("══════════════════════════════════════════════════")
EOF

echo ""
echo "[cost-report] Full JSON report: ${REPORT_FILE}"

# ── Compare to previous month ─────────────────────────────────────────────────
PREV_MONTH=$(date -d "${START} -1 month" +%Y-%m 2>/dev/null || \
             python3 -c "import datetime; d=datetime.date(${MONTH:0:4},${MONTH:5:2},1); prev=d.replace(day=1)-datetime.timedelta(days=1); print(prev.strftime('%Y-%m'))")
PREV_REPORT="${REPORT_DIR}/cost-report-${PREV_MONTH}.json"

if [[ -f "$PREV_REPORT" ]]; then
  echo "[cost-report] Month-over-month comparison:"
  python3 - <<EOF
import json

def total(path):
    with open(path) as f:
        data = json.load(f)
    results = data.get("ResultsByTime", [{}])[0]
    return sum(float(g["Metrics"]["BlendedCost"]["Amount"]) for g in results.get("Groups", []))

curr = total("${REPORT_FILE}")
prev = total("${PREV_REPORT}")
delta = curr - prev
pct = (delta / prev * 100) if prev > 0 else 0
arrow = "↑" if delta > 0 else "↓"
print(f"  Previous month: \${prev:.2f}")
print(f"  Current month:  \${curr:.2f}")
print(f"  Change:         {arrow} \${abs(delta):.2f} ({pct:+.1f}%)")
EOF
fi
