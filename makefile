#!/bin/env make
package.tar.gz: package/covert_c2_ping_server package/build.py package/covert_c2_ping.tar
	-rm package.tar.gz
	tar cvzf package.tar.gz package/covert_c2_ping_server package/build.py package/covert_c2_ping.tar

package/covert_c2_ping.tar: Dockerfile .dockerignore Cargo.lock Cargo.toml covert_c2_ping_client/src/* covert_c2_ping_common/src/*
	mkdir -p package
	cargo vendor
	docker build -t covert_c2_ping:latest .
	docker save covert_c2_ping:latest > package/covert_c2_ping.tar

package/build.py: build.py
	mkdir -p package
	-rm package/build.py
	cp build.py package/build.py

package/covert_c2_ping_server: dst/covert_c2_ping_server
	mkdir -p package
	-rm package/covert_c2_ping_server
	cp dst/covert_c2_ping_server package/covert_c2_ping_server

dst/covert_c2_ping_server: covert_c2_ping_server/src/* covert_c2_ping_common/src/*
	cargo build --release -p covert_c2_ping_server

clean:
	rm -rf dst package target vendor package.tar.gz