FROM rust:1.75-slim-buster as builder

RUN apt-get update && apt-get install -y libudev-dev clang pkg-config libssl-dev build-essential cmake protobuf-compiler

RUN update-ca-certificates

WORKDIR /usr/src/app

COPY . .

RUN --mount=type=cache,id=s/0f0a0f09-fa16-40b7-a98f-575b363605e4-/home/root/app/target,target=/home/root/app/target \
    --mount=type=cache,id=s/0f0a0f09-fa16-40b7-a98f-575b363605e4-/usr/local/cargo/registry,target=/usr/local/cargo/registry \
	cargo build --release --bin stakenet-keeper

#########

FROM debian:buster as validator-history
RUN apt-get update && apt-get install -y ca-certificates
ENV APP="stakenet-keeper"

COPY --from=builder /usr/src/app/target/release/$APP ./$APP

ENTRYPOINT ./$APP
