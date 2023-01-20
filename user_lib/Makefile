build:
	@python3 build.py
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/00hello_world -O binary target/riscv64gc-unknown-none-elf/release/00hello_world.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/01store_fault -O binary target/riscv64gc-unknown-none-elf/release/01store_fault.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/02power -O binary target/riscv64gc-unknown-none-elf/release/02power.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/03priv_inst -O binary target/riscv64gc-unknown-none-elf/release/03priv_inst.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/04priv_csr -O binary target/riscv64gc-unknown-none-elf/release/04priv_csr.bin
env:
	@rustup install nightly
	@rustup default nightly
	@rustup target add riscv64gc-unknown-none-elf
	@cargo install cargo-binutils
	@rustup component add llvm-tools-preview
	@rustup component add rust-src
