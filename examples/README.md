# Examples

A stage-first example suite for Axo, organized by stage and subfeature.

## Stage Directories

- `examples/initializer/**`
- `examples/scanner/**`
- `examples/parser/**`
- `examples/resolver/**`
- `examples/generator/**`
- `examples/runtime/**`
- `examples/overall/**`

Each subfeature contains:

- `pass/*.axo` -> should compile with no `error:` diagnostics.
- `fail/*.axo` -> should emit at least one `error:` diagnostic.

## Naming

- Folder names are lowercase.
- Multi-word folders and files use underscores (`_`).
- File names are descriptive and stage-relevant.

## Run

```bash
./examples/test_examples.sh
```

The runner auto-detects common LLVM 18 install roots and sets `LLVM_SYS_181_PREFIX` when needed.
