build:
	cargo build

cp_to_bin:
	cp target/debug/esker /opt/homebrew/bin

install_locally: build cp_to_bin

build_linux:
	cargo zigbuild --release --target x86_64-unknown-linux-gnu
