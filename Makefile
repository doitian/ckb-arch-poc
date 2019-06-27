server:
	@which watchexec &>/dev/null || ( echo "Requires watchexec" && exit 1 )
	cargo build
	RUST_LOG=ckb_arch_poc=trace,tokio=debug watchexec -i target -r target/debug/ckb-arch-poc

client:
	nc 127.0.0.1 12345
