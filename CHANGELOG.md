<!-- next-header -->

## [Unreleased] - ReleaseDate

### Changed

- Upgrade dependencies.

## [0.0.11] - 2023-08-04

### Changed

- Third party Tor v0.4.7.14 container image. (linux/amd64, linux/arm64)

## [0.0.10] - 2023-07-26

### Changed

- Upgrade dependencies.

## [0.0.9] - 2023-07-15

### Added

- Expose affinity, image pull secrets, node selector, tolerations and topology spread constraints configuration.
- Expose output parameter for CLI onion-key generate.

### Fixed

- TorIngress ownership references for OnionBalance, OnionService and HorizontalPodAutoscaler.
- OnionBalance, OnionKey, OnionService and TorIngress Custom Resource Definition additional printer columns.

## [0.0.8] - 2023-07-13

### Changed

- Upgrade dependencies.

## [0.0.7] - 2023-07-10

### Changed

- Upgrade dependencies.

## [0.0.6] - 2023-07-09

### Added

- Exposed full annotation and label configuration.

## [0.0.5] - 2023-07-08

### Added

- Exposed full container resource management configuration.

## [0.0.4] - 2023-07-07

### Added

- TorIngress HorizontalPodAutoscaler spec.
- OnionBalance, OnionKey, OnionService and TorIngress status conditions.
- Recreate Pods when ConfigYaml, Hostname, OBConfig or Torrc changes.

### Removed

- OnionBalance, OnionKey, OnionService and TorIngress status state.

## [0.0.3] - 2023-07-06

### Added

- OnionBalance, OnionService and TorIngress configurable resources.
- TorIngress Custom Resource Definition scale subresource.
- TorIngress HorizontalPodAutoscaler support.

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

[Unreleased]: https://github.com/agabani/tor-operator/compare/v0.0.11...HEAD
[0.0.11]: https://github.com/agabani/tor-operator/compare/v0.0.10...v0.0.11
[0.0.10]: https://github.com/agabani/tor-operator/compare/v0.0.9...v0.0.10
[0.0.9]: https://github.com/agabani/tor-operator/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/agabani/tor-operator/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/agabani/tor-operator/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/agabani/tor-operator/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/agabani/tor-operator/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/agabani/tor-operator/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/agabani/tor-operator/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/agabani/tor-operator/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/agabani/tor-operator/compare/e5f4f5d8a63d3ef610629b7575a188aca79d58cd...v0.0.1
