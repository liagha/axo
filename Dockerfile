FROM docker.iranserver.com/library/debian:sid-slim

WORKDIR /axo

COPY axo     ./axo
COPY examples ./examples

ENTRYPOINT ["./axo"]
