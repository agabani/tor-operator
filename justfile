set shell := ["bash", "-uc"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

GIT_COMMIT := `git rev-parse --verify --short HEAD`

# help
help:
  @just --list

# docker build
docker-build:
  docker build \
    --tag agabani/tor-operator:{{GIT_COMMIT}} \
    .

# docker buildx build
docker-buildx-build:
  docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --tag agabani/tor-operator:{{GIT_COMMIT}} \
    .

# docker build
docker-run: docker-build
  @docker run --rm agabani/tor-operator:{{GIT_COMMIT}}

# kube clean
kube-clean:
  @tilt down

# kube run
kube-run:
  @tilt up

# run crd
run-crd-generate:
  @cargo run -- crd generate -o ./helm/templates/crd.yaml
