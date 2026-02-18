FROM ubuntu:24.04 AS build

ARG LIBEVENT_SHA=92e6de1be9ec176428fd2367677e61ceffc2ee1cb119035037a27d346b0403bb
ARG LIBEVENT_VERSION=2.1.12

ARG OPENSSL_SHA=b1bfedcd5b289ff22aee87c9d600f515767ebf45f77168cb6d64f231f518a82e
ARG OPENSSL_VERSION=3.6.1

ARG TOR_SHA=c949c2f86b348e64891976f6b1e49c177655b23df97193049bf1b8cd3099e179
ARG TOR_VERSION=0.4.9.5

ARG ZLIB_SHA=bb329a0a2cd0274d05519d61c667c062e06990d72e125ee2dfa8de64f0119d16
ARG ZLIB_VERSION=1.3.2

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    asciidoc \
    automake \
    build-essential \
    curl

# download dependencies
RUN curl -fsSLO "https://www.openssl.org/source/openssl-${OPENSSL_VERSION}.tar.gz" && \
    echo "${OPENSSL_SHA}  openssl-${OPENSSL_VERSION}.tar.gz" | sha256sum -c && \
    tar zxvf openssl-${OPENSSL_VERSION}.tar.gz && \
    rm openssl-${OPENSSL_VERSION}.tar.gz

RUN curl -fsSLO "https://github.com/libevent/libevent/releases/download/release-${LIBEVENT_VERSION}-stable/libevent-${LIBEVENT_VERSION}-stable.tar.gz" && \
    echo "${LIBEVENT_SHA}  libevent-${LIBEVENT_VERSION}-stable.tar.gz" | sha256sum -c && \
    tar zxvf libevent-${LIBEVENT_VERSION}-stable.tar.gz && \
    rm libevent-${LIBEVENT_VERSION}-stable.tar.gz

RUN curl -fsSLO "https://zlib.net/zlib-${ZLIB_VERSION}.tar.gz" && \
    echo "${ZLIB_SHA}  zlib-${ZLIB_VERSION}.tar.gz" | sha256sum -c && \
    tar zxvf zlib-${ZLIB_VERSION}.tar.gz && \
    rm zlib-${ZLIB_VERSION}.tar.gz

RUN curl -fsSLO "https://dist.torproject.org/tor-${TOR_VERSION}.tar.gz" && \
    echo "${TOR_SHA}  tor-${TOR_VERSION}.tar.gz" | sha256sum -c && \
    tar zxvf tor-${TOR_VERSION}.tar.gz && \
    rm tor-${TOR_VERSION}.tar.gz

# build dependencies
RUN cd openssl-${OPENSSL_VERSION} && \
    ./config --prefix=$PWD/install --libdir=lib no-shared no-dso && \
    make -j$(nproc) && \
    make install && \
    cd ..

RUN cd libevent-${LIBEVENT_VERSION}-stable && \
    ./configure CFLAGS="-I/openssl-${OPENSSL_VERSION}/install/include" LDFLAGS="-L/openssl-${OPENSSL_VERSION}/install/lib" --prefix=$PWD/install --disable-shared --enable-static --with-pic && \
    make -j$(nproc) && \
    make install && \
    cd ..

RUN cd zlib-${ZLIB_VERSION} && \
    ./configure --prefix=$PWD/install && \
    make -j$(nproc) && \
    make install && \
    cd ..

RUN cd tor-${TOR_VERSION} && \
    ./configure --prefix=$PWD/install --enable-static-tor \
    --with-libevent-dir=/libevent-${LIBEVENT_VERSION}-stable/install \
    --with-openssl-dir=/openssl-${OPENSSL_VERSION}/install \
    --with-zlib-dir=/zlib-${ZLIB_VERSION}/install &&  \
    make -j$(nproc) && \
    make install && \
    cd ..

FROM ubuntu:24.04

ARG TOR_VERSION=0.4.9.5

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    gettext \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /tor-${TOR_VERSION}/LICENSE /licenses/tor/LICENSE
COPY --from=build /tor-${TOR_VERSION}/install/bin/tor /usr/local/bin/tor

CMD ["/usr/local/bin/tor"]
