#!/bin/env make

64client = target/x86_64-pc-windows-gnu/release/covert_c2_ping_client.exe
32client = target/i686-pc-windows-gnu/release/covert_c2_ping_client.exe
server = target/x86_64-unknown-linux-musl/release/covert_c2_ping_server
web = covert_c2_ping_web/dist

all: $(64client) $(32client) $(server) $(web)/index.html
	mkdir -p out
	mkdir -p out/artifact
	rm -rf out/static
	mkdir -p out/static
	cp $(64client) out/artifact/artifact_64
	cp $(32client) out/artifact/artifact_32
	cp $(server) out/covert_c2_ping
	cp $(web)/* out/static
	mdbook build
	rm -f covert_c2_ping.tar.gz
	tar -cvzf covert_c2_ping.tar.gz out/*

deploy: all
	ssh $(REMOTE) rm -rf out covert_c2_ping.tar.gz
	scp covert_c2_ping.tar.gz $(REMOTE):~/covert_c2_ping.tar.gz
	ssh $(REMOTE) tar -xvzf covert_c2_ping.tar.gz

clean:
	cargo clean
	rm -rf out
	rm -rf covert_c2_ping_web/dist
	rm -f covert_c2_ping.tar.gz

# -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort
$(32client): covert_c2_ping_client/src/*
	RUSTFLAGS='-C link-arg=-s' cargo build --target i686-pc-windows-gnu --release -p covert_c2_ping_client

$(64client): covert_c2_ping_client/src/*
	RUSTFLAGS='-C link-arg=-s' cargo build --target x86_64-pc-windows-gnu --release -p covert_c2_ping_client

$(server): covert_c2_ping_server/src/* covert_c2_ping_server/src/workers/*
	cross build --target x86_64-unknown-linux-gnu --release -p covert_c2_ping_server

$(web)/index.html: covert_c2_ping_web/src/*
	trunk build covert_c2_ping_web/index.html