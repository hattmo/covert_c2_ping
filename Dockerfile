FROM quay.io/pypa/manylinux_2_28_x86_64
RUN yum install -y clang
RUN yum install -y nftables-devel