FROM ubuntu:22.04 as build

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    asciidoc \
    automake \
    build-essential \
    curl

# download dependencies
RUN curl -fsSLO "https://github.com/libevent/libevent/releases/download/release-2.1.8-stable/libevent-2.1.8-stable.tar.gz" && \
    echo "965cc5a8bb46ce4199a47e9b2c9e1cae3b137e8356ffdad6d94d3b9069b71dc2  libevent-2.1.8-stable.tar.gz" | sha256sum -c && \
    tar zxvf libevent-2.1.8-stable.tar.gz && \
    rm libevent-2.1.8-stable.tar.gz

RUN curl -fsSLO "https://www.openssl.org/source/openssl-1.1.1u.tar.gz" && \
    echo "e2f8d84b523eecd06c7be7626830370300fbcc15386bf5142d72758f6963ebc6  openssl-1.1.1u.tar.gz" | sha256sum -c && \
    tar zxvf openssl-1.1.1u.tar.gz && \
    rm openssl-1.1.1u.tar.gz

RUN curl -fsSLO "https://zlib.net/zlib-1.2.13.tar.gz" && \
    echo "b3a24de97a8fdbc835b9833169501030b8977031bcb54b3b3ac13740f846ab30  zlib-1.2.13.tar.gz" | sha256sum -c && \
    tar zxvf zlib-1.2.13.tar.gz && \
    rm zlib-1.2.13.tar.gz

# build dependencies
RUN cd libevent-2.1.8-stable && \
    ./configure --prefix=$PWD/install --disable-shared --enable-static --with-pic && \
    make -j$(nproc) && \
    make install && \
    cd ..

RUN cd openssl-1.1.1u && \
    ./config --prefix=$PWD/install no-shared no-dso && \
    make -j$(nproc) && \
    make install && \
    cd ..

RUN cd zlib-1.2.13 && \
    ./configure --prefix=$PWD/install && \
    make -j$(nproc) && \
    make install && \
    cd ..

# download tor
RUN curl -fsSLO "https://dist.torproject.org/tor-0.4.7.13.tar.gz" && \
    echo "2079172cce034556f110048e26083ce9bea751f3154b0ad2809751815b11ea9d  tor-0.4.7.13.tar.gz" | sha256sum -c && \
    tar zxvf tor-0.4.7.13.tar.gz && \
    rm tor-0.4.7.13.tar.gz

# build tor
RUN cd tor-0.4.7.13 && \
    ./configure --prefix=$PWD/install --enable-static-tor \
    --with-libevent-dir=$PWD/../libevent-2.1.8-stable/install \
    --with-openssl-dir=$PWD/../openssl-1.1.1u/install \
    --with-zlib-dir=$PWD/../zlib-1.2.13/install &&  \
    make -j$(nproc) && \
    make install && \
    cd ..

FROM ubuntu:22.04
# FROM gcr.io/distroless/static-debian11

COPY --from=build /tor-0.4.7.13/LICENSE /licenses/tor/LICENSE
COPY --from=build /tor-0.4.7.13/install/bin/tor /usr/local/bin/tor

CMD ["/usr/local/bin/tor"]
