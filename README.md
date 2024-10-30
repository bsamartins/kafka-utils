```
awslocal kafka create-cluster \
--cluster-name "EventsCluster" \
--broker-node-group-info file://brokernodegroupinfo.json \
--kafka-version "2.8.0" \
--number-of-broker-nodes 3

rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-gnu

brew install cargo-zigbuild  
cargo zigbuild --target x86_64-unknown-linux-musl
```
---
```
rustup target add x86_64-unknown-linux-gnu
cargo install cargo-zigbuild
export PATH="$PATH;/usr/local/cargo/bin/"
cargo-zigbuild zigbuild --target x86_64-unknown-linux-gnu   

cargo-zigbuild zigbuild --target x86_64-unknown-linux-musl
OPENSSL_DIR
```
