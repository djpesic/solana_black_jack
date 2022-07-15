#!/bin/bash

function build_bpf() {
	cd client
	cargo build
	cd ../dealer
	cargo build
	cd ../utils
	cargo build
	cd ../player
	cargo build
	cd ..
    cargo build-bpf --manifest-path=program/Cargo.toml --bpf-out-dir=dist/program
}

case $1 in
    "build")
	build_bpf
	;;
    "deploy")
	build_bpf
	solana program deploy dist/program/black_jack.so
	;;
    "dealer")
	(cd dealer/; cargo run ../dist/program/black_jack-keypair.json)
	;;
	"player")
	(cd player/; cargo run ../dist/program/black_jack-keypair.json)
	;;
    "clean")
	(cd program/; cargo clean)
	(cd client/; cargo clean)
	(cd player/; cargo clean)
	rm -rf dist/
	;;
    *)
	echo "usage: $0 [build|clean|client]"
	echo "build: compilation"
	echo "clean: remove build products"
	;;
esac
