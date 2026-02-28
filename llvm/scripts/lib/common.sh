#!/usr/bin/env bash
set -euo pipefail

LLVM_VERSION="${LLVM_VERSION:-19.1.7}"
LLVM_TAG="llvmorg-${LLVM_VERSION}"
LLVM_REPO_URL="${LLVM_REPO_URL:-https://github.com/llvm/llvm-project.git}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${SCRIPTS_ROOT}/.." && pwd)"

WORK_DIR="${WORK_DIR:-/tmp/axo-llvm-build}"
LLVM_SRC_DIR="${LLVM_SRC_DIR:-${WORK_DIR}/llvm-project}"
LLVM_BUILD_DIR="${LLVM_BUILD_DIR:-${WORK_DIR}/llvm-build}"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64) echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *)
      echo "Unsupported architecture: $(uname -m)" >&2
      exit 1
      ;;
  esac
}

detect_os() {
  case "$(uname -s)" in
    Linux) echo "linux" ;;
    Darwin) echo "macos" ;;
    *)
      echo "Unsupported OS: $(uname -s)" >&2
      exit 1
      ;;
  esac
}

detect_jobs() {
  if command -v nproc >/dev/null 2>&1; then
    nproc
  else
    sysctl -n hw.ncpu
  fi
}

TARGET="${TARGET:-$(detect_arch)-$(detect_os)}"
JOBS="${JOBS:-$(detect_jobs)}"
DEST_DIR="${DEST_DIR:-${REPO_ROOT}/${TARGET}}"

print_context() {
  echo "LLVM version: ${LLVM_VERSION}"
  echo "Target: ${TARGET}"
  echo "Work directory: ${WORK_DIR}"
  echo "LLVM source: ${LLVM_SRC_DIR}"
  echo "LLVM build: ${LLVM_BUILD_DIR}"
  echo "Destination: ${DEST_DIR}"
}
