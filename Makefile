run_cli:
	@cd git-flow-rs-cli && cargo run

run_gui:
	@cd git-flow-rs-gui && cargo run
	
clean:
	@cargo clean
	@rm -Rf *.pkg.tar.zst git-flow-rs pkg src dist

release: clean
	@python resources/scripts/release.py

version:
	@python resources/scripts/set_version.py

test:
	@RUST_BACKTRACE=1 cargo test -- --no-capture --test-threads=1