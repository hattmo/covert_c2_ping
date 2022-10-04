FROM ubuntu:latest
SHELL [ "/bin/bash","-c" ]
RUN apt update && apt upgrade -y && apt install -y curl gcc-mingw-w64 build-essential
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default nightly
RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /root/working
ADD . .
RUN mkdir .cargo
RUN echo -e "\
    \n\
    [unstable]\n\
    unstable-options = true\n\
    [source.crates-io]\n\
    replace-with = \"vendored-sources\"\n\
    [source.vendored-sources]\n\
    directory = \"vendor\"\n\
    [build]\n\
    out-dir = \"dst\"\n\
    " >> ./.cargo/config.toml
RUN echo -e "\n\
[workspace]\n\
members = [\"covert_c2_ping_client\",\"covert_c2_ping_common\"]\n\
" > Cargo.toml

ENTRYPOINT [ "cargo","build","--release","--offline","-p","covert_c2_ping_client" ]
