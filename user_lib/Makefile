build:
	@cargo build --release
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/00power_3 -O binary target/riscv64gc-unknown-none-elf/release/00power_3.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/01power_5 -O binary target/riscv64gc-unknown-none-elf/release/01power_5.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/02power_7 -O binary target/riscv64gc-unknown-none-elf/release/02power_7.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/03sleep -O binary target/riscv64gc-unknown-none-elf/release/03sleep.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/04load_fault -O binary target/riscv64gc-unknown-none-elf/release/04load_fault.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/05store_fault -O binary target/riscv64gc-unknown-none-elf/release/05store_fault.bin
env:
	@rustup install nightly
	@rustup default nightly
	@rustup target add riscv64gc-unknown-none-elf
	@cargo install cargo-binutils
	@rustup component add llvm-tools-preview
	@rustup component add rust-src
