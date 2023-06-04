FROM ubuntu:22.04 as build

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    asciidoc \
    automake \
    build-essential \
    git \
    libevent-dev \
    libssl-dev \
    zlib1g-dev

RUN git clone https://git.torproject.org/tor.git

WORKDIR /tor

RUN git checkout tor-0.4.7.13

RUN ./autogen.sh && ./configure && make

FROM ubuntu:22.04

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    libevent-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /tor/src/app/tor /usr/bin

CMD ["tor"]
