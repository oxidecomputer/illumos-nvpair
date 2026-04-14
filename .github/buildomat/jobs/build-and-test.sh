#!/bin/bash
#:
#: name = "build-and-test"
#: variety = "basic"
#: target = "helios-latest"
#: rust_toolchain = "stable"
#: output_rules = []
#:

set -o errexit
set -o pipefail
set -o xtrace

cargo --version
rustc --version

banner fmt
cargo fmt --check

banner build
ptime -m cargo build --all-targets

banner test
cargo install cargo-nextest --locked
ptime -m cargo nextest run
