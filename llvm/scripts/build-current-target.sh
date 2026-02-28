#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

stages=(
  "01-check-tools.sh"
  "02-clone-llvm.sh"
  "03-configure.sh"
  "04-build.sh"
  "05-package.sh"
)

for stage in "${stages[@]}"; do
  echo "==> Running stage: ${stage}"
  "${SCRIPT_DIR}/stages/${stage}"
done
