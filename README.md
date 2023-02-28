# Covert C2 Ping

Covert C2 Ping is a implementation of a command and control tunnel for cobaltstrike
 over icmp Echo requests and responses.  The reference implementation for external C2
 can be found on [helpsystems.com](https://hstechdocs.helpsystems.com/manuals/cobaltstrike/current/userguide/content/topics/listener-infrastructue_external-c2.htm?cshid=1043)

 # Build Requirements

- nightly

```bash
rustup default nightly
```

- components

```bash
rustup component add rust-src
```

- targets

```bash
rustup target add x86_64-unknown-linux-musl
rustup target add i686-pc-windows-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add wasm32-unknown-unknown
```

- rust binaries

```
cargo install mdbook
cargo install trunk
```

- cross compiler
```
sudo apt install mingw-w64
```


# Runtime dependencies

The server requires nftables to be installed to function.

```bash
sudo apt update && sudo apt install nftables
```