#!/bin/sh

MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri)
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup set profile minimal
echo "$MIRI_NIGHTLY" > rust-toolchain
rustup component add miri

# cargo miri test
