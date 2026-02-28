# axo-llvm Build Agent Prompt

## Context

`axo-llvm` is a repository that contains pre-built static LLVM libraries for the Axo compiler.
The Axo compiler (https://github.com/liagha/axo) uses inkwell 0.8.0 with LLVM 19 and needs
static LLVM libs to build without requiring contributors to have LLVM installed on their system.

This repo ships those pre-built static libs for each supported platform. Each platform has its
own directory. The Axo build.rs points at the correct subdirectory for the current target.

## Repository Structure

```
axo-llvm/
  x86_64-linux/
    lib/        ← static .a files
    include/    ← llvm headers
  aarch64-linux/
    lib/
    include/
  x86_64-macos/
    lib/
    include/
  aarch64-macos/
    lib/
    include/
  ... (one directory per target)
```

## Your Job

You are on a machine with a specific OS and architecture. Your job is to:

1. Build LLVM 19 static libraries for this machine's target
2. Place the output in the correct directory in this repo
3. Commit and push

## Step 1 — Install Dependencies

**Linux (apt):**
```bash
sudo apt install cmake ninja-build clang git
```

**Linux (pacman/Manjaro):**
```bash
sudo pacman -S cmake ninja clang git
```

**macOS:**
```bash
brew install cmake ninja llvm git
```

## Step 2 — Determine Your Target Name

Your target directory name follows this pattern: `{arch}-{os}`

| Machine | Directory name |
|---|---|
| Linux x86_64 | `x86_64-linux` |
| Linux aarch64 | `aarch64-linux` |
| macOS x86_64 (Intel) | `x86_64-macos` |
| macOS aarch64 (Apple Silicon) | `aarch64-macos` |
| Android aarch64 | `aarch64-android` |

Run this to confirm your arch and os:
```bash
uname -m && uname -s
```

## Step 3 — Clone LLVM 19

```bash
git clone --depth 1 --branch llvmorg-19.1.7 https://github.com/llvm/llvm-project.git
```

## Step 4 — Build

```bash
mkdir llvm-build
cd llvm-build

cmake ../llvm-project/llvm \
  -G Ninja \
  -DCMAKE_BUILD_TYPE=MinSizeRel \
  -DCMAKE_C_COMPILER=clang \
  -DCMAKE_CXX_COMPILER=clang++ \
  -DLLVM_TARGETS_TO_BUILD="X86;AArch64" \
  -DLLVM_ENABLE_PROJECTS="" \
  -DLLVM_BUILD_LLVM_DYLIB=OFF \
  -DLLVM_LINK_LLVM_DYLIB=OFF \
  -DLLVM_BUILD_TOOLS=OFF \
  -DLLVM_BUILD_TESTS=OFF \
  -DLLVM_ENABLE_TERMINFO=OFF \
  -DLLVM_ENABLE_ZLIB=OFF \
  -DLLVM_ENABLE_ZSTD=OFF \
  -DLLVM_INCLUDE_BENCHMARKS=OFF \
  -DLLVM_INCLUDE_EXAMPLES=OFF

cmake --build . -j$(nproc)
```

**Note:** Do not use GCC to build. Always use clang. GCC 15+ has compatibility issues with LLVM 19 headers.

## Step 5 — Package Into the Repo

Clone the axo-llvm repo if you haven't already:
```bash
git clone https://github.com/liagha/axo-llvm.git
```

Then copy the built files into the correct target directory:
```bash
TARGET="x86_64-linux"   # change this to your target

mkdir -p axo-llvm/$TARGET/lib
mkdir -p axo-llvm/$TARGET/include

cp llvm-build/lib/*.a axo-llvm/$TARGET/lib/
cp -r llvm-build/include/llvm axo-llvm/$TARGET/include/
```

Verify size:
```bash
du -sh axo-llvm/$TARGET
```

Expected: roughly 150-200MB uncompressed.

## Step 6 — Commit and Push

```bash
cd axo-llvm
git add $TARGET/
git commit -m "add llvm-19 static $TARGET"
git push
```

Use exactly this commit message format: `add llvm-19 static {target}`

## Rules

- Always use LLVM 19.1.7 exactly. Do not use a different version.
- Always use clang to compile, never gcc.
- Always use `MinSizeRel` build type, never `Release` or `Debug`.
- Only copy `.a` files from `lib/`. Do not copy `.so`, `.dylib`, or cmake files.
- Only copy `include/llvm/` headers. Do not copy `include/llvm-c/` if it doesn't exist — that's fine.
- Do not modify other target directories. Only add your own.
- Do not force push.
- The directory name must exactly match the target pattern above.

## Verification

After copying, confirm the lib directory contains `.a` files:
```bash
ls axo-llvm/$TARGET/lib/*.a | head -5
```

And the include directory contains llvm headers:
```bash
ls axo-llvm/$TARGET/include/llvm | head -5
```

Both should return results. If either is empty something went wrong.
