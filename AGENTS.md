# AGENTS.md

## Purpose
This repository contains **Axo**, a Rust compiler for the `.axo` language.  
Your job is to make correct, minimal, testable changes across the compiler pipeline.

## Project Snapshot
- Language: Rust (`edition = 2021`)
- Binary crate: `axo`
- Entry point: `src/main.rs`
- Pipeline stages:
  1. `initializer` (CLI preferences + input discovery)
  2. `scanner` (tokenization)
  3. `parser` (AST/symbol formation)
  4. `resolver` (analysis + type checking)
  5. `generator` (LLVM IR + executable via `inkwell`)
- Built-in base library used by tests: `base/option.axo`

## Working Rules
- Prefer small, targeted edits over large refactors.
- Preserve existing module boundaries and naming conventions.
- Do not change behavior outside the requested scope.
- If changing diagnostics, keep existing error style (`error:`) compatible with example tests.

## Commands
- Build: `cargo build`
- Run compiler (example): `target/debug/axo -i base/option.axo -i test.axo`
- Run example suite: `./examples/test_examples.sh`

## CLI Behavior (Current)
Initializer recognizes these preferences:
- `-i` / `-input` for input files
- implicit path arguments as input
- `-o` / `-output` for output path
- `-o.ir` / `-o.ll` for IR output path
- `-o.exec` / `-o.executable` for binary output path
- `-r` / `-run` to execute produced binary
- `-v` / `-verbose` and `-q` / `-quiet` for verbosity

## Test Layout
Examples are behavior specs:
- `examples/<stage>/<feature>/pass/**/case.axo`
- `examples/<stage>/<feature>/fail/**/case.axo`

`examples/test_examples.sh` classifies a case as:
- `pass`: no `error:` in compiler output
- `fail`: at least one `error:` in compiler output

When changing scanner/parser/resolver behavior, validate against the relevant `examples/**` paths.

## Coding Guidance
- Follow existing style in neighboring files.
- Reuse existing helpers (`Registry`, `Reporter`, stage `execute` methods) before adding new abstractions.
- Keep lifetimes/ownership changes conservative; this codebase relies on explicit lifetime flow across stages.
- Add comments only when logic is non-obvious.

## Delivery Checklist
For each change:
1. Explain what changed and why.
2. List touched files.
3. Run relevant checks (`cargo build`) and report results.
4. Call out any untested areas or assumptions.
