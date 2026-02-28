#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ -z "${LLVM_SYS_181_PREFIX:-}" ]]; then
  for candidate in \
    "/usr/lib/llvm18" \
    "/usr/lib/llvm-18" \
    "/usr/local/lib/llvm18" \
    "/usr/local/opt/llvm@18" \
    "/opt/homebrew/opt/llvm@18"
  do
    if [[ -d "$candidate" ]]; then
      export LLVM_SYS_181_PREFIX="$candidate"
      break
    fi
  done
fi

cargo build >/dev/null
bin="target/debug/axo"

if [[ ! -x "$bin" ]]; then
  echo "missing compiler binary at $bin"
  exit 1
fi

base_files=("$repo_root/base/option.axo")
mismatches=0
pass_total=0
fail_total=0
case_timeout="${AXO_EXAMPLE_TIMEOUT:-10}"

run_compiler() {
  local args=("$@")

  # Prevent examples like read_line() from blocking the suite waiting for TTY input.
  if command -v timeout >/dev/null 2>&1; then
    printf '\n' | timeout "${case_timeout}s" "$bin" "${args[@]}" 2>&1
  elif command -v gtimeout >/dev/null 2>&1; then
    printf '\n' | gtimeout "${case_timeout}s" "$bin" "${args[@]}" 2>&1
  else
    printf '\n' | "$bin" "${args[@]}" 2>&1
  fi
}

run_case() {
  local expected="$1"
  local file="$2"

  local args=()
  for base in "${base_files[@]}"; do
    args+=("-i" "$base")
  done
  args+=("-i" "$file")

  local output
  local status
  set +e
  output="$(run_compiler "${args[@]}" | sed -E 's/\x1B\[[0-9;]*[[:alpha:]]//g')"
  status=$?
  set -e

  if [[ "$status" -eq 124 ]]; then
    output="${output}"$'\nerror: timed out while running example case.'
  elif [[ "$status" -ne 0 ]]; then
    output="${output}"$'\nerror: compiler process exited with non-zero status.'
  fi

  local actual
  if printf '%s' "$output" | rg -q "error:"; then
    actual="fail"
  else
    actual="pass"
  fi

  if [[ "$actual" == "$expected" ]]; then
    printf 'ok   [%s] %s
' "$expected" "$file"
    return 0
  fi

  mismatches=$((mismatches + 1))
  printf 'bad  [expected=%s actual=%s] %s
' "$expected" "$actual" "$file"
  printf '%s
' "$output" | tail -n 20
  return 0
}

while IFS= read -r file; do
  pass_total=$((pass_total + 1))
  run_case pass "$file"
done < <(find examples -type f -name '*.axo' -path '*/pass/*' | sort)

while IFS= read -r file; do
  fail_total=$((fail_total + 1))
  run_case fail "$file"
done < <(find examples -type f -name '*.axo' -path '*/fail/*' | sort)

total=$((pass_total + fail_total))
printf '\nSummary: total=%d pass_cases=%d fail_cases=%d mismatches=%d
' "$total" "$pass_total" "$fail_total" "$mismatches"

if [[ "$mismatches" -ne 0 ]]; then
  exit 1
fi
