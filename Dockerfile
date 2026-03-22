FROM docker.iranserver.com/library/debian:sid-slim

RUN rm -f /etc/apt/sources.list.d/debian.sources && \
    echo "deb http://mirror.shatel.ir/debian sid main" > /etc/apt/sources.list

RUN apt-get -o Acquire::Check-Valid-Until=false update && apt-get install -y \
    llvm-19 \
    llvm-19-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

ENV LLVM_SYS_191_PREFIX="/usr/lib/llvm-19"
ENV LD_LIBRARY_PATH="/usr/lib/llvm-19/lib:${LD_LIBRARY_PATH}"

WORKDIR /axo

COPY axo /axo/axo
COPY runtime /axo/runtime
COPY examples /axo/examples

ENTRYPOINT ["/axo/axo"]
