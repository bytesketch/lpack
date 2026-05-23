.PHONY: build help clean install

help:
	@echo "|==========================================================|"
	@echo "|          lpack makefile                                  |"
	@echo "|==========================================================|"
	@echo "| Run 'make build'   to build.                             |"
	@echo "| Run 'make install' to install                            |"
	@echo "| Run 'make clean'   to remove caches and build.           |"
	@echo "|----------------------------------------------------------|"
	@echo "| Run 'make build install' .            (one-line command) |"
	@echo "|==========================================================|"

build:
	@echo "Building..."
	@cargo build --release
	@mkdir -p target/output
	@echo "Build successful."

clean:
	@echo "Cleaning..."
	@rm -rf target
	@rm -rf lpack
	@echo "Cleaning successful."

install:
	@echo "Installing... (sudo needed)"
	@./target/release/lpack build .
	@sudo ./target/release/lpack install lpack/build/lpack-1.0-beta.lpk --system-wide
