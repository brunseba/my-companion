#!/usr/bin/env sh
# Extracts one version's section from CHANGELOG.md, stripping the heading
# itself. Understands both heading shapes commit-and-tag-version writes:
#   ## 1.2.3 (2026-08-16)                      (no prior tag to link to)
#   ## [1.2.3](compare-url) (2026-08-16)         (once there is one)
#
# Usage: changelog-section.sh <version> [changelog-file]
#   changelog-section.sh 0.1.1
set -eu

version="$1"
file="${2:-CHANGELOG.md}"

awk -v ver="$version" '
  /^## / {
    if (flag) exit
    line = $0
    gsub(/\[|\]\([^)]*\)/, "", line)   # "[1.2.3](url)" -> "1.2.3"
    if (line ~ "^## " ver " ") { flag = 1 }
    next
  }
  flag { print }
' "$file" | awk 'NF{p=1} p'
