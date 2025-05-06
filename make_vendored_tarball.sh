#!/usr/bin/env bash
set -euo pipefail

APP_NAME="myapp"
VERSION="0.1"
ROOT_DIR="${APP_NAME}-${VERSION}"
TARBALL="${ROOT_DIR}.tar.gz"

echo "[*] Cleaning up any old vendor directories..."
rm -rf ${ROOT_DIR}
rm -f ${TARBALL}

echo "[*] Copying source tree..."
mkdir -p ${ROOT_DIR}
cp main.c tokenizer.json Makefile ${ROOT_DIR}/

echo "[*] Copying Rust crate..."
mkdir -p ${ROOT_DIR}/rust_tokenizer
cp -r rust_tokenizer/src ${ROOT_DIR}/rust_tokenizer/
cp rust_tokenizer/Cargo.toml rust_tokenizer/Cargo.lock ${ROOT_DIR}/rust_tokenizer/

echo "[*] Vendoring Rust dependencies..."
cd ${ROOT_DIR}/rust_tokenizer
cargo vendor vendor/
mkdir -p .cargo
cat > .cargo/config.toml <<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF
cd ../../

echo "[*] Creating tarball: ${TARBALL}"
tar czf ${TARBALL} ${ROOT_DIR}

echo "[✓] Done. You can now build with:"
echo "    tar xf ${TARBALL} && cd ${ROOT_DIR} && cargo build --offline --release"
