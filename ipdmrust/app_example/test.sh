#!/bin/sh
RUST_BACKTRACE=1 cargo test --release --target=x86_64-unknown-linux-gnu -- --test-threads=1 $@
