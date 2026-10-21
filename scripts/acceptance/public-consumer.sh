#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
consumer="$(mktemp -d)"
trap 'rm -rf -- "$consumer"' EXIT
mkdir "$consumer/source" "$consumer/tests"
git -C "$root" archive HEAD | tar -xf - -C "$consumer/source"
cp "$root/scripts/acceptance/public-consumer.rs" "$consumer/tests/public.rs"
cat > "$consumer/Cargo.toml" <<TOML
[package]
name = "public-snapshot-consumer"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
zixcel-container-registry = { path = "$consumer/source" }
serde_json = "1.0"
TOML
cp "$root/Cargo.lock" "$consumer/Cargo.lock"
CARGO_TARGET_DIR="$root/target" cargo generate-lockfile --manifest-path "$consumer/Cargo.toml" --offline
CARGO_TARGET_DIR="$root/target" cargo test --manifest-path "$consumer/Cargo.toml" --locked --offline
