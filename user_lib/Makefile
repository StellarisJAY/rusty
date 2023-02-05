elf:
	@cargo build --release
env:
	@rustup install nightly
	@rustup default nightly
	@rustup target add riscv64gc-unknown-none-elf
	@cargo install cargo-binutils
	@rustup component add llvm-tools-preview
	@rustup component add rust-src
