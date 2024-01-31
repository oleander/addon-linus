FROM rust:latest as builder

ARG SUPERVISOR_TOKEN
ARG OPENAI_API_KEY
ARG SOCKET_PORT

ENV SUPERVISOR_TOKEN=${SUPERVISOR_TOKEN}
ENV OPENAI_API_KEY=${OPENAI_API_KEY}
ENV SOCKET_PORT=${SOCKET_PORT}
ENV CARGO_BUILD_TARGET_DIR=/target

WORKDIR /app

RUN cargo install cargo-build-deps

COPY Cargo.* /app/
RUN cargo build-deps --release

COPY src/ /app/src/
RUN cargo build --bin websocket --release

FROM rust:slim

RUN apt-get update && apt-get install -y netcat-openbsd
COPY --from=builder /target/*/websocket /usr/bin/
HEALTHCHECK CMD echo "ping" | nc -w 1 localhost ${SOCKET_PORT} || exit 1

WORKDIR /app
COPY . /app

ENTRYPOINT websocket
