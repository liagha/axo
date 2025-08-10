compile path:
    cargo run -- -v -p {{path}}

generate path:
    cargo run --features generator -- -v -p {{path}}

llvm-fix:
    export LLVM_SYS_181_PREFIX=/opt/local/libexec/llvm-18/

lib-fix: llvm-fix
    export LIBRARY_PATH="/opt/local/lib:${LIBRARY_PATH:-}"
    export CPATH="/opt/local/include:${CPATH:-}"

bitcode-to-object path out:
    clang -c {{path}} -o {{out}}

rust-to-object path out:
    rustc --emit=obj {{path}} -o {{out}}

link-two-objects path1 path2 out:
    clang {{path1}} {{path2}} -o {{out}}

clear-lab:
    rm -rf lab/*