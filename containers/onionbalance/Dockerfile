FROM ubuntu:22.04 as build

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    git \
    python3 \
    python3-setuptools

RUN git clone https://gitlab.torproject.org/tpo/core/onionbalance.git

WORKDIR /onionbalance

RUN git fetch && git checkout 0.2.2

RUN python3 setup.py install

FROM ubuntu:22.04

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    python3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/lib/python3.10/ /usr/local/lib/python3.10/
COPY --from=build /usr/local/bin/onionbalance /usr/local/bin

CMD ["onionbalance"]
