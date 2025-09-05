run path:
    cargo run --features generator -- -v -p {{path}}
    llc-mp-18 -filetype=obj lab/test.ll -o lab/input.o
    clang lab/input.o -o lab/exec
    lab/exec