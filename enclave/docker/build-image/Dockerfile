
ARG UBUNTU_VERSION=22.04

# compiler version is fixed for mrenclave
FROM rust:1.82 as henchman-rust

RUN rustup update
SHELL ["/bin/bash", "-c"]

RUN curl -fsSLo /usr/share/keyrings/gramine-keyring.gpg https://packages.gramineproject.io/gramine-keyring.gpg && \
    echo 'deb [arch=amd64 signed-by=/usr/share/keyrings/gramine-keyring.gpg] https://packages.gramineproject.io/ jammy main' > /etc/apt/sources.list.d/gramine.list

RUN apt-get update && apt-get install -y apt-utils curl ca-certificates git build-essential wget libssl-dev git unzip pkgconf
RUN apt install -y pkg-config
RUN apt install -y gramine
RUN gramine-sgx-gen-private-key

RUN git clone https://github.com/daltoncoder/Henchman.git /opt/henchman

WORKDIR /opt/henchman
RUN git checkout main

WORKDIR /opt/henchman
RUN cargo build --release
RUN mkdir -p enclave/gramine/bin
RUN cp target/release/tee_ai_agent enclave/gramine/bin/

WORKDIR /opt/henchman/enclave/gramine
RUN cd trusted && ./update-trusted.sh && cd ../
RUN make SGX=1 SGX_DEV_BUILD=0 all

# ----------------------------------------------------------------

FROM ubuntu:${UBUNTU_VERSION} as henchman-gramine
SHELL ["/bin/bash", "-c"]

ARG CODE_VERSION
ENV CODE_VERSION=${CODE_VERSION:-"v0.1.0"}

ARG ENCLAVE_VERBOSITY
ENV ENCLAVE_VERBOSITY=${ENCLAVE_VERBOSITY:-3}
ENV VERSION_TAG="${CODE_VERSION}-${CHAIN}"

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y curl tzdata apt-utils ca-certificates libssl-dev reprepro lsb-release

RUN curl -fsSLo /usr/share/keyrings/gramine-keyring.gpg https://packages.gramineproject.io/gramine-keyring.gpg && \
    echo 'deb [arch=amd64 signed-by=/usr/share/keyrings/gramine-keyring.gpg] https://packages.gramineproject.io/ jammy main' > /etc/apt/sources.list.d/gramine.list && \
    curl -fsSLo /usr/share/keyrings/intel-sgx-deb.key https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key && \
    echo 'deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx-deb.key] https://download.01.org/intel-sgx/sgx_repo/ubuntu jammy main' > /etc/apt/sources.list.d/intel-sgx.list

    curl -fsSLo /etc/apt/keyrings/gramine-keyring-$(lsb_release -sc).gpg https://packages.gramineproject.io/gramine-keyring-$(lsb_release -sc).gpg
    echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/gramine-keyring-$(lsb_release -sc).gpg] https://packages.gramineproject.io/ $(lsb_release -sc) main" \
    | tee /etc/apt/sources.list.d/gramine.list
    curl -fsSLo /etc/apt/keyrings/intel-sgx-deb.asc https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key
    echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/intel-sgx-deb.asc] https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -sc) main" \
    | tee /etc/apt/sources.list.d/intel-sgx.list
    
    
RUN apt-get update && \
    wget http://archive.ubuntu.com/ubuntu/pool/main/p/protobuf/libprotobuf17_3.6.1.3-2ubuntu5_amd64.deb && \
    dpkg -i libprotobuf17_3.6.1.3-2ubuntu5_amd64.deb && \
    apt-get install -y --no-install-recommends tzdata && \
    apt install -y sgx-aesm-service libsgx-aesm-launch-plugin libsgx-aesm-quote-ex-plugin libsgx-aesm-ecdsa-plugin libsgx-dcap-quote-verify libsgx-dcap-ql libsgx-quote-ex libsgx-dcap-default-qpl-dev

RUN gramine-sgx-gen-private-key

WORKDIR /opt/henchman/enclave/gramine
COPY --from=henchman-rust /opt/henchman/enclave/gramine ./

EXPOSE 443
ENTRYPOINT ["gramine-sgx", "tee_ai_agent"]

LABEL version=$VERSION_TAG
