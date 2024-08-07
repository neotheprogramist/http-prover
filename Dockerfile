FROM rust:1-alpine AS chef
# Use apk for package management in Alpine
RUN apk add --no-cache build-base libressl-dev
RUN cargo install cargo-chef

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo build --release -p prover
RUN cargo install --git https://github.com/lambdaclass/cairo-vm --rev 37ea72977dccbc2b90b8b7534c1edabd2e2fef79 cairo1-run


FROM piotr439/prover:latest AS prover


FROM python:3.9.18-slim-bookworm AS final
RUN apt update && apt install -y build-essential libgmp-dev elfutils jq git
RUN pip install --upgrade pip

COPY --from=builder /app/target/release/prover /usr/local/bin
COPY --from=builder /usr/local/cargo/bin/cairo1-run /usr/local/bin/cairo1-run
COPY --from=prover /usr/bin/cpu_air_prover /usr/local/bin/cpu_air_prover

RUN git clone --depth=1 -b v2.7.0-rc.3 https://github.com/starkware-libs/cairo.git 
RUN mv cairo/corelib/ . 
RUN rm -rf cairo

RUN pip install cairo-lang==0.13.1
RUN pip install sympy==1.7.1


ENV HOST=0.0.0.0 \
    PORT=3000 \
    JWT_SECRET_KEY=jwt \
    MESSAGE_EXPIRATION_TIME=3600 \
    SESSION_EXPIRATION_TIME=3600 \
    AUTHORIZED_KEYS="0xda78e64363a0a924b3d4d292c0b0926d64b78e8c9e372c37f63f3a7b20af9842"

EXPOSE 3000


ENTRYPOINT [ "prover" ]