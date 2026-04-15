.PHONY: build bridge server install uninstall clean

build: bridge server

bridge:
	cd bridge && cargo build --release

server:
	cargo build -p simbridge-server --release

install:
	./install.sh

uninstall:
	./uninstall.sh

clean:
	cargo clean
	cd bridge && cargo clean
