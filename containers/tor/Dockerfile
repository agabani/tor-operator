FROM ubuntu:26.04 AS build

ARG LIBEVENT_SHA=f7e9383b8c0baa81b687e5b5eecc01beefaf1b19b64151d95ed61647fe7a315c
ARG LIBEVENT_VERSION=2.1.13

ARG OPENSSL_SHA=736b467530f916737b7031310ccb21d8218c6229e61e8e160cd1d3458cd543a8
ARG OPENSSL_VERSION=4.0.2

ARG TOR_SHA=c0d307c9dcdaee4848a8ca53e9d6c4ec92823e4f30be12790b0fbddfc6515f5b
ARG TOR_VERSION=0.4.9.12

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

FROM ubuntu:26.04

ARG TOR_VERSION=0.4.9.12

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    gettext \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /tor-${TOR_VERSION}/LICENSE /licenses/tor/LICENSE
COPY --from=build /tor-${TOR_VERSION}/install/bin/tor /usr/local/bin/tor

CMD ["/usr/local/bin/tor"]
