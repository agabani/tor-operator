FROM ubuntu:24.04 AS build

ARG LIBEVENT_SHA=92e6de1be9ec176428fd2367677e61ceffc2ee1cb119035037a27d346b0403bb
ARG LIBEVENT_VERSION=2.1.12

ARG OPENSSL_SHA=002a2d6b30b58bf4bea46c43bdd96365aaf8daa6c428782aa4feee06da197df3
ARG OPENSSL_VERSION=3.4.1

ARG TOR_SHA=5d5d99e21992c4c71af1afcef16c70f4c5e7ee021633ac138b2a2761be75064b
ARG TOR_VERSION=0.4.8.15

ARG ZLIB_SHA=9a93b2b7dfdac77ceba5a558a580e74667dd6fede4585b91eefb60f03b72df23
ARG ZLIB_VERSION=1.3.1

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

ARG TOR_VERSION=0.4.8.15

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    gettext \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /tor-${TOR_VERSION}/LICENSE /licenses/tor/LICENSE
COPY --from=build /tor-${TOR_VERSION}/install/bin/tor /usr/local/bin/tor

CMD ["/usr/local/bin/tor"]
