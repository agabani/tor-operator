FROM ubuntu:22.04 as build

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    asciidoc \
    automake \
    build-essential \
    curl

# download dependencies
RUN curl -fsSLO "https://www.openssl.org/source/openssl-1.1.1v.tar.gz" && \
    echo "d6697e2871e77238460402e9362d47d18382b15ef9f246aba6c7bd780d38a6b0  openssl-1.1.1v.tar.gz" | sha256sum -c && \
    tar zxvf openssl-1.1.1v.tar.gz && \
    rm openssl-1.1.1v.tar.gz

RUN curl -fsSLO "https://github.com/libevent/libevent/releases/download/release-2.1.12-stable/libevent-2.1.12-stable.tar.gz" && \
    echo "92e6de1be9ec176428fd2367677e61ceffc2ee1cb119035037a27d346b0403bb  libevent-2.1.12-stable.tar.gz" | sha256sum -c && \
    tar zxvf libevent-2.1.12-stable.tar.gz && \
    rm libevent-2.1.12-stable.tar.gz

RUN curl -fsSLO "https://zlib.net/zlib-1.3.tar.gz" && \
    echo "ff0ba4c292013dbc27530b3a81e1f9a813cd39de01ca5e0f8bf355702efa593e  zlib-1.3.tar.gz" | sha256sum -c && \
    tar zxvf zlib-1.3.tar.gz && \
    rm zlib-1.3.tar.gz

RUN curl -fsSLO "https://dist.torproject.org/tor-0.4.7.14.tar.gz" && \
    echo "a5ac67f6466380fc05e8043d01c581e4e8a2b22fe09430013473e71065e65df8  tor-0.4.7.14.tar.gz" | sha256sum -c && \
    tar zxvf tor-0.4.7.14.tar.gz && \
    rm tor-0.4.7.14.tar.gz

# build dependencies
RUN cd openssl-1.1.1v && \
    ./config --prefix=$PWD/install no-shared no-dso && \
    make -j$(nproc) && \
    make install && \
    cd ..

RUN cd libevent-2.1.12-stable && \
    ./configure CFLAGS="-I/openssl-1.1.1v/install/include" LDFLAGS="-L/openssl-1.1.1v/install/lib" --prefix=$PWD/install --disable-shared --enable-static --with-pic && \
    make -j$(nproc) && \
    make install && \
    cd ..

RUN cd zlib-1.3 && \
    ./configure --prefix=$PWD/install && \
    make -j$(nproc) && \
    make install && \
    cd ..

RUN cd tor-0.4.7.14 && \
    ./configure --prefix=$PWD/install --enable-static-tor \
    --with-libevent-dir=/libevent-2.1.12-stable/install \
    --with-openssl-dir=/openssl-1.1.1v/install \
    --with-zlib-dir=/zlib-1.3/install &&  \
    make -j$(nproc) && \
    make install && \
    cd ..

FROM ubuntu:22.04

COPY --from=build /tor-0.4.7.14/LICENSE /licenses/tor/LICENSE
COPY --from=build /tor-0.4.7.14/install/bin/tor /usr/local/bin/tor

CMD ["/usr/local/bin/tor"]
