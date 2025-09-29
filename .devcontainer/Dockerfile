FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install clang busybox protobuf-compiler mold \
   && apt-get upgrade -y \
   && apt-get autoremove -y && apt-get clean -y \
   && rm -rf /var/lib/apt/lists/*
# Set build argument for architecture
ARG TARGETARCH
RUN apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends wget \
    # Set GOARCH and download URL based on TARGETARCH
    && if [ "$TARGETARCH" = "arm64" ]; then \
        GO_TARBALL="go1.24.1.linux-arm64.tar.gz"; \
      else \
        GO_TARBALL="go1.24.1.linux-amd64.tar.gz"; \
      fi \
    && wget https://go.dev/dl/${GO_TARBALL} \
    && rm -rf /usr/local/go \
    && tar -C /usr/local -xzf ${GO_TARBALL} \
    && rm -f ${GO_TARBALL} \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /usr/local/gopath
ENV GOPATH="/usr/local/gopath"
ENV GOBIN="/usr/local/gopath/bin"
ENV PATH="/usr/local/go/bin:/usr/local/gopath/bin:${PATH}"
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"


RUN go install github.com/ktr0731/evans@latest \
    && go install github.com/bufbuild/buf/cmd/buf@latest \
    && go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest \
    && go install github.com/fullstorydev/grpcui/cmd/grpcui@latest
    
USER vscode
ENV CARGO_HOME="/home/vscode/.cargo"
RUN mkdir /home/vscode/.cargo
RUN cargo install tokei sea-orm-cli
