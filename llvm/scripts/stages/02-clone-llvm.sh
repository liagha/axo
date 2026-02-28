#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib/common.sh
source "${SCRIPT_DIR}/../lib/common.sh"

mkdir -p "${WORK_DIR}"

if [[ ! -d "${LLVM_SRC_DIR}/.git" ]]; then
  echo "Cloning LLVM source (${LLVM_TAG})..."
  git clone --depth 1 --branch "${LLVM_TAG}" "${LLVM_REPO_URL}" "${LLVM_SRC_DIR}"
else
  echo "LLVM source already exists at ${LLVM_SRC_DIR}; reusing it."
fi
