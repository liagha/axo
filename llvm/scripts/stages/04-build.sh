#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib/common.sh
source "${SCRIPT_DIR}/../lib/common.sh"

echo "Building static libraries with ${JOBS} jobs..."
cmake --build "${LLVM_BUILD_DIR}" -j"${JOBS}"
