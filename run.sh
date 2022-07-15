#!/bin/bash

function build_bpf() {
	cd clients
	cargo build
	cd ..
    cargo build-bpf --manifest-path=program/Cargo.toml --bpf-out-dir=program/dist/program
}

case $1 in
    "build")
	build_bpf
	;;
    "deploy")
	build_bpf
	solana program deploy program/dist/program/black_jack.so
	;;
    "dealer")
	(./clients/target/debug/dealer program/dist/program/black_jack-keypair.json)
	;;
	"player")
	(./clients/target/debug/player program/dist/program/black_jack-keypair.json)
	;;
    "clean")
	(cd clients/; cargo clean)
	(cd program/; cargo clean)
	rm -rf program/dist/
	;;
    *)
	echo "usage: $0 [build|clean|client]"
	echo "build: compilation"
	echo "clean: remove build products"
	;;
esac
