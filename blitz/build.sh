#!/usr/bin/env bash
set -euo pipefail
shopt -s inherit_errexit
shopt -s extglob

rm -r pkg/ out/ || true
mkdir out/ || true
mkdir pkg/

if ! [ -d "blitz" ]; then
	(
		git clone https://github.com/dioxuslabs/blitz
		cd blitz || exit 1
		git reset --hard 15fe8ccfbc99cf7254ccfaa6c46e5a1c3a2c1cb8
		git apply ../blitz.patch
	)
fi

if ! [ -d "stylo" ]; then
	(
		git clone https://github.com/servo/stylo
		cd stylo || exit 1
		git reset --hard v0.11.0
		git apply ../stylo.patch
	)
fi

if [ "${MINIMAL:-0}" = "1" ]; then
	CARGOFLAGS="--no-default-features"
else
	CARGOFLAGS=""
fi

WBG="wasm-bindgen 0.2.108"
if [ "$(wasm-bindgen -V)" != "$WBG" ]; then
	echo "Incorrect wasm-bindgen version: '$(wasm-bindgen -V)' != '$WBG'"
	exit 1
fi

export CFLAGS='-O3' 
cargo build --target wasm32-unknown-unknown -Z build-std=panic_abort,std -Z build-std-features=optimize_for_size --release $CARGOFLAGS "$@"
echo "[wbg] cargo finished"
wasm-bindgen --target web --out-dir out/ target/wasm32-unknown-unknown/release/blitz_dl.wasm
echo "[wbg] wasm-bindgen finished"

if ! [ "${RELEASE:-0}" = "1" ]; then
	: "${WASMOPTFLAGS:=-g}"
else
	: "${WASMOPTFLAGS:=}"
fi

mv out/blitz_dl_bg.wasm out/blitz_dl_unoptimized.wasm

if [ "${RELEASE:-0}" = "1" ]; then
	time wasm-opt $WASMOPTFLAGS \
		out/blitz_dl_unoptimized.wasm -o out/blitz_dl_bg.wasm \
		-O4 -O4 -O4 -Oz -Oz -Oz -O4
else
	mv out/blitz_dl_unoptimized.wasm out/blitz_dl_bg.wasm
fi
echo "[wbg] wasm-opt finished"

cp out/blitz_dl.{js,d.ts} pkg/
cp out/blitz_dl_bg.wasm pkg/blitz_dl.wasm

rm -r out/
echo "[wbg] done!"
