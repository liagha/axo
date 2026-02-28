#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib/common.sh
source "${SCRIPT_DIR}/../lib/common.sh"

echo "Copying artifacts to ${DEST_DIR}..."
mkdir -p "${DEST_DIR}/lib" "${DEST_DIR}/include"
cp "${LLVM_BUILD_DIR}"/lib/*.a "${DEST_DIR}/lib/"
rm -rf "${DEST_DIR}/include/llvm"
cp -R "${LLVM_BUILD_DIR}/include/llvm" "${DEST_DIR}/include/"

echo "Verifying output..."
ls "${DEST_DIR}"/lib/*.a | head -5
ls "${DEST_DIR}"/include/llvm | head -5
du -sh "${DEST_DIR}"

echo "Done."
echo "Suggested commit command:"
echo "  git add ${TARGET}/ && git commit -m \"add llvm-19 static ${TARGET}\""
