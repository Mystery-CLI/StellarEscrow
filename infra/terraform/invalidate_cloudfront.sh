#!/usr/bin/env bash
set -euo pipefail
if [ $# -lt 2 ]; then
  echo "Usage: $0 <distribution-id> <path1> [path2 ...]"
  exit 1
fi
DIST_ID="$1"; shift
aws cloudfront create-invalidation --distribution-id "$DIST_ID" --paths "$@"
