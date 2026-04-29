# Combinator Framework Refactor Plan

## Goal
Turn the current combinator engine into a reusable framework while keeping the **existing public constructor APIs** stable (especially the function signatures currently exposed through `Formation` and `operation`).

## Current pain points observed
- `Formation` currently mixes parser state, tree state, control flow outcome, and composition helpers in one place.
- `Former` hardcodes memoization storage (`memo`) and token/form pushing (`push`) alongside orchestration.
- Caching exists as a data shape (`Memo`, `Record`) but not as an independent pluggable behavior.
- Atomized use-cases (grep/sed-like text pipelines, keybinding matcher) want reusable micro-combinators and execution policies.

## Architectural split (without breaking signatures)

### 1) Keep `Formation` as the stable DSL facade
`Formation::{literal, predicate, sequence, alternative, optional, repetition, ...}` should remain the user-facing DSL entry point.

Internally, `Formation` should only carry:
- combinator identity
- compositional combinator object
- minimal runtime cursor snapshot

Move non-core runtime concerns behind traits.

### 2) Introduce an execution context trait
Create a trait (example):

- `ExecutionContext`:
  - push consumed input/form nodes
  - read/write cursor state
  - emit failure/panic
  - optionally memoize

Then make `Former` one concrete implementation of `ExecutionContext`.

This lets you keep current function signatures while delegating behavior.

### 3) Move caching into a policy combinator
Convert memoization from hardcoded map mutation in executor into a **combinator wrapper**:

- `with_cache(strategy)` or `memoized(key_fn, store)`.

Behavior:
1. Before evaluating child combinator, consult cache backend.
2. On hit: restore outcome + record.
3. On miss: run child, snapshot result, persist.

This keeps cache optional, composable, and testable.

### 4) Split input/form mutation from execution driver
Current `push` logic should be moved into a tiny focused component:

- `InputSink` (or `FormBuilder`) responsible only for:
  - advancing source
  - appending consumed item
  - creating leaf `Form::input`
  - updating formation-local references

`Former::build` should orchestrate; mutation helpers should be delegated.

### 5) Normalize outcomes into combinator-level contracts
You already have good `Outcome` semantics (`Aligned`, `Failed`, `Panicked`, etc.).

Codify this into combinator contracts:
- Matchers: produce `Aligned|Blank|Failed`
- Structural combinators: escalate child outcomes
- Effect combinators (`transform`, `recover`, `panic`): may emit failures and force terminal states

This gives deterministic behavior for atomic reuse.

## Suggested module layout

```text
src/combinator/
  core/
    formation.rs       # stable API surface
    combinator.rs      # trait + dyn wrappers
    outcome.rs
  runtime/
    context.rs         # ExecutionContext trait
    engine.rs          # build/dispatch
    sink.rs            # push/input-tree mutation
  policies/
    cache.rs           # memo store trait + memoized combinator
    recovery.rs
  primitives/
    matchers.rs        # literal/predicate/etc.
    structure.rs       # sequence/alternative/repetition
    effects.rs         # transform/fail/panic/recover/ignore/skip
  adapters/
    operation.rs       # bridges existing operation module API
```

## Compatibility strategy (important)
To preserve current API signatures:

1. Keep existing `Formation` methods and `operation` constructors unchanged.
2. Re-route internals to new modules.
3. Provide temporary compatibility shims in old files calling new implementations.
4. Deprecate internals in phases; do not deprecate user-facing signatures.

## Phase plan

### Phase 1: Mechanical extraction (no behavior change)
- Extract `Outcome` and runtime helpers into dedicated modules.
- Extract `push` into `InputSink` and call it from `Former`.
- Keep tests green.

### Phase 2: Introduce cache abstraction
- Define `MemoStore` trait with default in-memory store.
- Implement `NoCache` and `HashMemoStore`.
- Add `memoized` combinator wrapper; migrate existing memo usage behind it.

### Phase 3: Atomize combinators
- Split current multi-purpose combinators into:
  - pure matchers
  - pure structural composition
  - pure effects
- Ensure each combinator owns one reason to change.

### Phase 4: New verticals
Build proof-of-verticals using only public constructors:
- grep/sed-like pipeline: `find -> capture -> transform -> replace`.
- keybinding matcher: sequence + alternatives + repetition + timeout/policy combinators.

## Practical first refactor to do next
Start with the lowest-risk/highest-value step:

1. Extract current `Former::push` into `runtime/sink.rs` (`InputSink::push_input`).
2. Replace direct body in `Former::push` with delegation.
3. Add small unit tests for sink behavior (consumed/form/stack indices).

This immediately improves aesthetics and atomization without touching public APIs.

## Design rule of thumb
If a component needs both to decide *what* to match and *how* to store execution artifacts, split it.
- combinator decides behavior
- policy decides storage/optimization
- runtime decides orchestration
