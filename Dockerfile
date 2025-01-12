FROM mcr.microsoft.com/devcontainers/rust:dev-1 AS guest_builder 

RUN apt-get update \
    && apt-get -y install \
        # build-essential \
        # cmake \
        # curl \
        # git \
        gnupg \
        # gnuplot \
        lsb-release \
        # make \
        software-properties-common \
        # sudo \
        wget

ARG LLVM_VERSION=17

RUN wget https://apt.llvm.org/llvm.sh \
    && chmod +x ./llvm.sh         \
    && sudo ./llvm.sh ${LLVM_VERSION} all      \
    && sudo ln -s /usr/lib/llvm-${LLVM_VERSION}/bin/clang-cl /usr/bin/clang-cl \
    && sudo ln -s /usr/lib/llvm-${LLVM_VERSION}/bin/llvm-lib /usr/bin/llvm-lib \
    && sudo ln -s /usr/lib/llvm-${LLVM_VERSION}/bin/lld-link /usr/bin/lld-link \
    && sudo ln -s /usr/lib/llvm-${LLVM_VERSION}/bin/llvm-ml /usr/bin/llvm-ml   \
    && sudo ln -s /usr/lib/llvm-${LLVM_VERSION}/bin/ld.lld /usr/bin/ld.lld     \
    && sudo ln -s /usr/lib/llvm-${LLVM_VERSION}/bin/clang /usr/bin/clang

ARG RUST_TOOLCHAIN=1.81.0

RUN rustup default ${RUST_TOOLCHAIN} \
    && rustup target add x86_64-unknown-none
    
WORKDIR /app

COPY guest/ .

RUN cargo build --release


FROM mcr.microsoft.com/devcontainers/rust:dev-1 AS host_builder 
WORKDIR /app
COPY host/ .
RUN cargo build --release


ENTRYPOINT ["my_rust_app"]
FROM mcr.microsoft.com/azurelinux/base/rust:1
COPY --from=guest_builder /app/target/x86_64-unknown-none/release/chick-guest /usr/local/bin/chick-guest
COPY --from=host_builder /app/target/release/chick /usr/local/bin/chick
ENTRYPOINT ["chick"]
