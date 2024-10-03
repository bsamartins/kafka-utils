FROM ubuntu

RUN apt update -y && \
    apt install -y curl build-essential snapd

RUN curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf > rustup_installer.sh
RUN chmod +x rustup_installer.sh
RUN bash -- ./rustup_installer.sh -y -v
RUN export PATH="~/.cargo/bin:$PATH"
RUN ~/.cargo/bin/rustup toolchain install stable

RUN ~/.cargo/bin/cargo install cargo-zigbuild
    # libc6-compat musl-dev openssl-dev

#    rustup target add x86_64-unknown-linux-musl \
RUN cargo install cargo-zigbuild
#    export PATH="$PATH;/usr/local/cargo/bin/"
