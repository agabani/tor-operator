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

# docker build tor
docker-build-tor:
  docker build \
    --tag agabani/tor:{{GIT_COMMIT}} \
    tor

# docker buildx build tor
docker-buildx-build-tor:
  docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --tag agabani/tor:{{GIT_COMMIT}} \
    tor

# docker build tor operator
docker-build-tor-operator:
  docker build \
    --tag agabani/tor-operator:{{GIT_COMMIT}} \
    .

# docker buildx build tor operator
docker-buildx-build-tor-operator:
  docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --tag agabani/tor-operator:{{GIT_COMMIT}} \
    .

# docker build
docker-run: docker-build-tor-operator
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
