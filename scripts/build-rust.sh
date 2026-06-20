#!/usr/bin/env bash
# Build the Rust core for Android (arm64) and drop the .so where Gradle picks it up.
#
# Requires: rustup target `aarch64-linux-android`, `cargo-ndk`, and an NDK on
# disk reachable via $ANDROID_NDK_HOME / $ANDROID_NDK_ROOT.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JNILIBS="$ROOT/app/src/main/jniLibs"

cd "$ROOT/rust"
cargo ndk --target arm64-v8a --output-dir "$JNILIBS" build --release

echo "Built: $JNILIBS/arm64-v8a/libspeedy_core.so"
