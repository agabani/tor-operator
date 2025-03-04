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

# kube example onion key
cli-onion-key-generate-example:
  @{{ if path_exists("./example/secrets/onionkey") == "true" { "" } else { "mkdir -p ./example/secrets/onionkey" } }}
  @{{ if path_exists("./example/secrets/onionkey/hostname") == "true" { "" } else { "cargo run -- onion-key generate --output ./example/secrets/onionkey" } }}
  @{{ if path_exists("./example/secrets/onionkey/hs_ed25519_public_key") == "true" { "" } else { "cargo run -- onion-key generate --output ./example/secrets/onionkey" } }}
  @{{ if path_exists("./example/secrets/onionkey/hs_ed25519_secret_key") == "true" { "" } else { "cargo run -- onion-key generate --output ./example/secrets/onionkey" } }}

# cli markdown generate
cli-markdown-generate:
  @cargo run -- markdown generate --output ./docs/cli_help.md

# cli onion key generate
cli-onion-key-generate:
  @cargo run -- onion-key generate

# docker build
docker-build: docker-build-onion-balance docker-build-tor docker-build-tor-operator

# docker buildx build
docker-buildx-build: docker-buildx-build-onion-balance docker-buildx-build-tor docker-buildx-build-tor-operator

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

# docker load
docker-load: docker-load-onion-balance docker-load-tor docker-load-tor-operator

# docker load onion balance
docker-load-onion-balance:
  @docker load --input agabani-onion-balance.tar

# docker load tor
docker-load-tor:
  @docker load --input agabani-tor.tar

# docker load tor-operator
docker-load-tor-operator:
  @docker load --input agabani-tor-operator.tar

# docker run onion balance
docker-run-onion-balance: docker-build-onion-balance
  @docker run --rm agabani/onion-balance:{{GIT_COMMIT}}

# docker run tor
docker-run-tor: docker-build-tor
  @docker run --rm agabani/tor:{{GIT_COMMIT}}

# docker run tor-operator
docker-run-tor-operator: docker-build-tor-operator
  @docker run --rm agabani/tor-operator:{{GIT_COMMIT}}

# docker save
docker-save: docker-save-onion-balance docker-save-tor docker-save-tor-operator

# docker save onion balance
docker-save-onion-balance:
  @docker save --output agabani-onion-balance.tar agabani/onion-balance:{{GIT_COMMIT}}

# docker save tor
docker-save-tor:
  @docker save --output agabani-tor.tar agabani/tor:{{GIT_COMMIT}}

# docker save tor-operator
docker-save-tor-operator:
  @docker save --output agabani-tor-operator.tar agabani/tor-operator:{{GIT_COMMIT}}

# generate
generate: cli-crd-generate cli-markdown-generate license

# kube clean
kube-clean:
  @tilt down

# kube load
kube-load:
  @kind load docker-image agabani/onion-balance:{{GIT_COMMIT}} agabani/tor:{{GIT_COMMIT}} agabani/tor-operator:{{GIT_COMMIT}}

# kube port-forward jaeger
kube-port-forward-jaeger:
  @kubectl -n jaeger port-forward service/jaeger-query 16686:16686

# kube port-forward hotrod
kube-port-forward-jaeger-hotrod:
  @kubectl -n jaeger port-forward service/jaeger-hotrod 16680:80

# kube run
kube-run: cli-onion-key-generate-example
  @tilt up

# kube test
kube-test:
  @helm upgrade tor-operator ./charts/tor-operator/ \
    --create-namespace \
    --install \
    --namespace tor-operator \
    --set onionBalance.image.repository=agabani/onion-balance \
    --set onionBalance.image.tag={{GIT_COMMIT}} \
    --set tor.image.repository=agabani/tor \
    --set tor.image.tag={{GIT_COMMIT}} \
    --set image.repository=agabani/tor-operator \
    --set image.tag={{GIT_COMMIT}}
  @helm test tor-operator --namespace tor-operator --timeout 15m0s

# license
license:
  @cargo bundle-licenses --format yaml --output docs/licenses/third_party.yaml

# lint
lint:
  @cargo clippy

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

# release dryrun
release-dryrun version: generate
  @cargo release {{version}}

# release execute
release-execute version: generate
  @cargo release {{version}} --execute --no-publish

# test unit
test-unit:
  @cargo test

# test e2e
test-e2e: docker-build kube-load kube-test
