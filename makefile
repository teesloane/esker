build:
	cargo build

cp_to_bin:
	cp target/debug/esker /opt/homebrew/bin

install_locally: build cp_to_bin

build_linux:
	cargo zigbuild --release --target x86_64-unknown-linux-gnu

build_mac_m1:
	cargo build --release

build_mac_intel:
	cargo build --release --target x86_64-apple-darwin

collect_builds:
	cp target/release/esker out/esker
	cd out; zip esker_mac_arm.zip esker
	rm out/esker

	cp target/x86_64-unknown-linux-gnu/release/esker out/esker
	cd out; zip esker_x86_64-unknown-linux-gnu.zip esker
	rm out/esker

	cp target/x86_64-apple-darwin/release/esker out/esker
	cd out; zip x86_64-apple-darwin.zip esker
	rm out/esker

build_all_and_collect: build_linux build_mac_m1 build_linux collect_builds

# == Test Site Cmds ==

example_site_remove:
	rm -rf tests/example_site/_esker/

example_site_new:
	cd tests/example_site && ../../target/debug/esker new

example_site_build:
	cd tests/example_site && ../../target/debug/esker build

example_site: example_site_remove example_site_new example_site_build

example_site_watch:
	cd tests/example_site && ../../target/debug/esker watch


### -- Running test ---

test_cli_overwrite:
	TRYCMD=overwrite cargo test --test cli_tests -- --nocapture
test_cli_dump:
	TRYCMD=dump cargo test --test cli_tests -- --nocapture

test_cli:
	cargo test --test cli_tests -- --nocapture
