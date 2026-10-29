#!/usr/bin/env bash
# Benchmark 2, file writes in Linux (WSL 2): the prototype and the final
# version measured with the fsync kind of bench/run.js (FsStore in Node.js
# and FileStorage of the native CLI) in alternating runs, as on Windows.
# Run in WSL from a clone of the repository whose origin is the Windows
# checkout; the WebAssembly packages are copied from the Windows builds in
# bench/versions/ (the build is the same on both systems), the CLI is built
# for x86_64-unknown-linux-musl with rust-lld (no C compiler in the system).
#
#   bash docs/research/data/benchmark-2/linux.sh <windows checkout> <out dir>
set -euo pipefail
WIN=${1:?windows checkout, e.g. /mnt/c/Users/me/dbms-rust-wasm}
OUT=${2:?output directory}
BASE=${BASE:-v1-prototyp}
FINAL=${FINAL:-v2-optymalizacja}
source ~/.nvm/nvm.sh
nvm use -s "${NODE_VERSION:-24.21.0}" > /dev/null # the Node.js of Benchmark 1 on Linux
source ~/.cargo/env
cd ~/flatsql-bench
git fetch -q origin --tags
git checkout -q --detach "$FINAL"
for v in "$BASE" "$FINAL"; do
  wt=~/b2/$v
  [ -d "$wt" ] || git worktree add -q --detach "$wt" "$v"
  (cd "$wt" && RUSTFLAGS='-C linker=rust-lld -C link-self-contained=yes' cargo build -q --release --target x86_64-unknown-linux-musl -p flatsql-cli)
  mkdir -p "$wt/target/release"
  cp "$wt/target/x86_64-unknown-linux-musl/release/flatsql" "$wt/target/release/flatsql"
  rm -rf "$wt/pkg" && cp -r "$WIN/bench/versions/$v/pkg" "$wt/pkg"
done
node bench/compare.js --before ~/b2/"$BASE" --after ~/b2/"$FINAL" --out "$OUT" --targets node,native --kinds fsync --runs 10
