#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib/common.sh
source "${SCRIPT_DIR}/../lib/common.sh"

require_cmd git
require_cmd cmake
require_cmd ninja
require_cmd clang
require_cmd clang++

print_context
echo "Toolchain checks passed."
