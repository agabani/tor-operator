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
  @cargo run -- crd generate --format helm --output ./charts/tor-operator/templates
  @cargo run -- crd generate --format yaml --output ./docs/custom_resource_definitions

# cli markdown generate
cli-markdown-generate:
  @cargo run -- markdown generate --output ./docs/cli/help.md

# cli onion key generate
cli-onion-key-generate:
  @cargo run -- onion-key generate

# docker build onion balance
docker-build-onion-balance:
  docker build \
    --tag agabani/onion-balance:{{GIT_COMMIT}} \
    ./containers/onion-balance

# docker buildx build onion balance
docker-buildx-build-onion-balance:
  docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --tag agabani/onion-balance:{{GIT_COMMIT}} \
    ./containers/onion-balance

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

# docker run onion balance
docker-run-onion-balance: docker-build-onion-balance
  @docker run --rm agabani/onion-balance:{{GIT_COMMIT}}

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

# kube dashboard port-forward
kube-dashboard-port-forward:
  @kubectl -n kubernetes-dashboard port-forward services/kubernetes-dashboard 8443:443

# kube dashboard token
kube-dashboard-token:
  @kubectl -n kubernetes-dashboard create token admin-user

# mkdocs build
mkdocs-build:
  @mkdocs build

# mkdocs install
mkdocs-install:
  @pip3 install mkdocs
  @pip3 install mkdocs-include-markdown-plugin

# mkdocs serve
mkdocs-serve:
  @mkdocs serve

# lint
lint:
  @cargo clippy
