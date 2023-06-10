set shell := ["bash", "-uc"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

GIT_COMMIT := `git rev-parse --verify --short HEAD`

# help
help:
  @just --list

# build
build:
  @cargo build --release

# cli controller run
cli-controller-run:
  @cargo run -- controller run

# cli crd generate
cli-crd-generate:
  @cargo run -- crd generate --output ./helm/templates

# docker build onionbalance
docker-build-onionbalance:
  docker build \
    --tag agabani/onionbalance:{{GIT_COMMIT}} \
    ./containers/onionbalance

# docker buildx build tor
docker-buildx-build-onionbalance:
  docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --tag agabani/onionbalance:{{GIT_COMMIT}} \
    ./containers/onionbalance

# docker build tor
docker-build-tor:
  docker build \
    --tag agabani/tor:{{GIT_COMMIT}} \
    ./containers/tor

# docker buildx build tor
docker-buildx-build-tor:
  docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --tag agabani/tor:{{GIT_COMMIT}} \
    ./containers/tor

# docker build tor-operator
docker-build-tor-operator:
  docker build \
    --tag agabani/tor-operator:{{GIT_COMMIT}} \
    .

# docker buildx build tor-operator
docker-buildx-build-tor-operator:
  docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --tag agabani/tor-operator:{{GIT_COMMIT}} \
    .

# docker run onionbalance
docker-run-onionbalance: docker-build-onionbalance
  @docker run --rm agabani/onionbalance:{{GIT_COMMIT}}

# docker run tor
docker-run-tor: docker-build-tor
  @docker run --rm agabani/tor-operator:{{GIT_COMMIT}}

# docker run tor-operator
docker-run-tor-operator: docker-build-tor-operator
  @docker run --rm agabani/tor-operator:{{GIT_COMMIT}}

# kube clean
kube-clean:
  @tilt down

# kube run
kube-run:
  @tilt up

# lint
lint:
  @cargo clippy
