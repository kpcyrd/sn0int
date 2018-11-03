#!/bin/sh
set -xue

# tested with rustc 1.30.0 and cargo 1.30.0

# by default, the build folder is located in /tmp, which is a tmpfs. The target/ folder
# can become quite large, causing the build to fail if we don't have enough RAM.
export TMPDIR="$HOME/tmp/repro-test"
mkdir -p "$TMPDIR"

# fileordering: https://github.com/diesel-rs/diesel/pull/1902
# build_path: https://github.com/briansmith/ring/issues/715

reprotest -vv --vary=-time,-domain_host,-fileordering,-build_path --source-pattern 'Cargo.* src/ sn0int-registry/ migrations/' '
    CARGO_HOME="$PWD/.cargo" RUSTUP_HOME='"$HOME/.rustup"' \
        RUSTFLAGS="--remap-path-prefix=$HOME=/remap-home --remap-path-prefix=$PWD=/remap-pwd" \
        cargo build --release --verbose --locked' \
    target/release/sn0int
