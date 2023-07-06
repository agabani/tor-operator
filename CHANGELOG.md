<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- OnionBalance, OnionKey, OnionService and TorIngress status conditions.

### Removed

- OnionBalance, OnionKey, OnionService and TorIngress status state.

## [0.0.3] - 2023-07-06

### Added

- OnionBalance, OnionService and TorIngress configurable resources.
- TorIngress Custom Resource Definition scale subresource.
- TorIngress Horizontal Pod Autoscaler support.

### Changed

- Use camel case for Custom Resource Definition schema.
- Use upper camel case for Custom Resource Definition.

## [0.0.2] - 2023-07-05

### Added

- OnionBalance, OnionKey, OnionService and TorIngress Custom Resource Definition additional printer columns.

## [0.0.1] - 2023-07-04

### Added

- OnionBalance, OnionKey, OnionService and TorIngress Custom Resource Definition.
- OnionBalance, OnionKey, OnionService and TorIngress controller.
- Tor Operator binary. (aarch64-macos, x86_64-linux, x86_64-macos, x86_64-windows)
- Tor Operator container image. (linux/amd64, linux/arm64)
- Tor Operator helm chart.
- Tor Operator observability. (logs, metrics)
- Third party Onion Balance v0.2.2 container image. (linux/amd64, linux/arm64)
- Third party Tor v0.4.7.13 container image. (linux/amd64, linux/arm64)

<!-- next-url -->

[Unreleased]: https://github.com/agabani/tor-operator/compare/v0.0.3...HEAD
[0.0.3]: https://github.com/agabani/tor-operator/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/agabani/tor-operator/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/agabani/tor-operator/compare/e5f4f5d8a63d3ef610629b7575a188aca79d58cd...v0.0.1
