build:
	cargo build

cp_to_bin:
	cp target/debug/esker /opt/homebrew/bin

install_locally: build cp_to_bin

build_linux:
	cargo zigbuild --release --target x86_64-unknown-linux-gnu


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
