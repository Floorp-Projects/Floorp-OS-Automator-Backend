FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install clang busybox protobuf-compiler \
   && wget https://go.dev/dl/go1.24.1.linux-amd64.tar.gz \
   && tar -C /usr/local -xzf go1.24.1.linux-amd64.tar.gz \
   && rm go1.24.1.linux-amd64.tar.gz \
   && apt-get autoremove -y && apt-get clean -y \
   && rm -rf /var/lib/apt/lists/*


RUN mkdir -p /usr/local/gopath
ENV GOPATH="/usr/local/gopath"
ENV GOBIN="/usr/local/gopath/bin"
ENV PATH="/usr/local/go/bin:/usr/local/gopath/bin:${PATH}"


RUN go install github.com/ktr0731/evans@latest \
    && go install github.com/bufbuild/buf/cmd/buf@latest
