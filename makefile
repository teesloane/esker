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

test_site_remove:
	rm -rf test_site/_esker/

test_site_new:
	cd test_site && ../target/debug/esker new

test_site_build:
	cd test_site && ../target/debug/esker build

test_site: test_site_remove test_site_new test_site_build

test_site_watch:
	cd test_site &&../target/debug/esker watch
