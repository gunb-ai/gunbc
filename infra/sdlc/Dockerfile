FROM rust:1.82-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release -p gunbc-app --bin gunbc-sdlc \
    && cargo build --release -p daglang-cli --bin daglang

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates git curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/gunbc-sdlc /usr/local/bin/gunbc-sdlc
COPY --from=builder /app/target/release/daglang /usr/local/bin/daglang

COPY dsl/ /app/dsl/
COPY target/codegen/ /app/codegen/

WORKDIR /app

ENV SDLC_LEDGER_DIR=/data/ledger
ENV SDLC_REPO_OWNER=gunb-ai
ENV SDLC_REPO_NAME=gunbc

VOLUME ["/data/ledger"]

ENTRYPOINT ["gunbc-sdlc"]
CMD ["worker", "--dry-run"]
