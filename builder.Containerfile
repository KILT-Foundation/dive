FROM debian:oldstable

RUN dpkg --add-architecture arm64
RUN apt-get update && apt-get install -y curl llvm-dev libclang-dev clang crossbuild-essential-arm64
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup install stable
RUN rustup target add aarch64-unknown-linux-gnu