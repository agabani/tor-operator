FROM ubuntu:24.04 AS build

ARG ONIONBALANCE_VERSION=0.2.4

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    git \
    python3 \
    python3-pip \
    python3-setuptools

RUN git clone https://gitlab.torproject.org/tpo/onion-services/onionbalance.git

WORKDIR /onionbalance

RUN git fetch && git checkout ${ONIONBALANCE_VERSION}

RUN python3 -m pip install . --break-system-packages

FROM ubuntu:24.04

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y \
    gettext \
    python3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/lib/python3.12/ /usr/local/lib/python3.12/
COPY --from=build /usr/local/bin/onionbalance /usr/local/bin

CMD ["onionbalance"]
