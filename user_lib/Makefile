build:
	@python3 build.py
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/00hello_world -O binary target/riscv64gc-unknown-none-elf/release/00hello_world.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/01store_fault -O binary target/riscv64gc-unknown-none-elf/release/01store_fault.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/02power -O binary target/riscv64gc-unknown-none-elf/release/02power.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/03priv_inst -O binary target/riscv64gc-unknown-none-elf/release/03priv_inst.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/04priv_csr -O binary target/riscv64gc-unknown-none-elf/release/04priv_csr.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/05power_3 -O binary target/riscv64gc-unknown-none-elf/release/05power_3.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/06power_5 -O binary target/riscv64gc-unknown-none-elf/release/06power_5.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/07power_7 -O binary target/riscv64gc-unknown-none-elf/release/07power_7.bin
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/08sleep -O binary target/riscv64gc-unknown-none-elf/release/08sleep.bin
env:
	@rustup install nightly
	@rustup default nightly
	@rustup target add riscv64gc-unknown-none-elf
	@cargo install cargo-binutils
	@rustup component add llvm-tools-preview
	@rustup component add rust-src
