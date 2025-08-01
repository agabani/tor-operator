<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.0.38] - 2025-06-20

### Added

- OpenTelemetry logs.
- OpenTelemetry metrics.
- OpenTelemetry console exporter.
- OpenTelemetry OTLP compression support.
- OpenTelemetry OTLP header support.
- OpenTelemetry OTLP SSL support.

### Removed

- `/metrics` endpoint.

## [0.0.37] - 2025-04-14

### Changed

- Third party Tor v0.4.8.16 container image with OpenSSL 3.5.0 and Ubuntu 24.10. (linux/amd64, linux/arm64)
- Third party Onion Balance v0.2.3 container image with Ubuntu 24.10. (linux/amd64, linux/arm64)
- Upgrade dependencies.

## [0.0.36] - 2025-03-22

### Changed

- Third party Tor v0.4.8.15 container image. (linux/amd64, linux/arm64)

## [0.0.35] - 2025-03-06

### Changed

- Upgrade dependencies.

## [0.0.34] - 2025-03-04

### Changed

- Third party Tor v0.4.8.14 container image with OpenSSL 3.4.1. (linux/amd64, linux/arm64)
- Third party Onion Balance v0.2.3 container image. (linux/amd64, linux/arm64)
- Upgrade dependencies.

## [0.0.33] - 2024-10-29

### Changed

- Third party Tor v0.4.8.13 container image with OpenSSL 3.4.0. (linux/amd64, linux/arm64)
- Upgrade dependencies.

## [0.0.32] - 2024-06-16

### Changed

- Third party Tor v0.4.8.12 container image with OpenSSL 3.3.1. (linux/amd64, linux/arm64)

## [0.0.31] - 2024-04-20

### Changed

- Third party Tor v0.4.8.11 container image with OpenSSL 3.3.0. (linux/amd64, linux/arm64)

## [0.0.30] - 2024-04-15

### Changed

- Third party Tor v0.4.8.11 container image. (linux/amd64, linux/arm64)

## [0.0.29] - 2024-04-08

### Added

- Exposed partial init container configuration.

### Changed

- `*.spec.**.deployment.containers` configuration from map to array.
- `*.spec.**.deployment.containers.*.env` configuration from map to array.
- `*.spec.**.deployment.containers.*.ports` configuration from map to array.
- `*.spec.**.deployment.containers.*.volumeMounts` configuration from map to array.

## [0.0.28] - 2024-04-06

### Changed

- Third party Tor v0.4.8.10 container image. (linux/amd64, linux/arm64)

## [0.0.27] - 2024-04-01

### Added

- Exposed partial volume configuration.

### Changed

- `*.spec.**.deployment.containers.*.env` configuration from array to map.
- `*.spec.**.deployment.containers.*.ports` configuration from array to map.
- `*.spec.**.deployment.containers.*.volumeMounts` configuration from array to map.

### Removed

- `*.spec.**.deployment.containers.*.envFrom`.

## [0.0.26] - 2024-03-29

### Added

- Exposed partial container configuration.
- Exposed torrc configuration.
- Distributed gettext in third party Onion Balance v0.2.2 container image. (linux/amd64, linux/arm64)
- Distributed gettext in third party Tor v0.4.8.9 container image. (linux/amd64, linux/arm64)

### Changed

- `*.spec.**.deployment.containers.onionBalance` renamed to `*.spec.**.deployment.containers.onionbalance`.

## [0.0.25] - 2024-03-05

- Upgrade dependencies.

## [0.0.24] - 2024-01-27

### Changed

- Upgrade dependencies.

## [0.0.23] - 2024-01-18

### Changed

- Upgrade dependencies.

## [0.0.22] - 2024-01-01

### Added

- Drop all Linux capabilities.
- Expose pod security context configuration.
- Run as non-root UID by default.

## [0.0.21] - 2023-12-31

### Changed

- Upgrade dependencies.

## [0.0.20] - 2023-11-20

### Changed

- Third party Tor v0.4.8.9 container image. (linux/amd64, linux/arm64)

## [0.0.19] - 2023-09-29

### Changed

- Third party Tor v0.4.8.7 container image. (linux/amd64, linux/arm64)

## [0.0.18] - 2023-09-21

### Changed

- Third party Tor v0.4.8.6 container image. (linux/amd64, linux/arm64)

## [0.0.17] - 2023-09-02

### Changed

- Third party Tor v0.4.8.5 container image. (linux/amd64, linux/arm64)

## [0.0.16] - 2023-08-27

### Added

- Tor Operator observability. (tracing)

## [0.0.15] - 2023-08-25

### Changed

- Third party Tor v0.4.8.4 container image. (linux/amd64, linux/arm64)

## [0.0.14] - 2023-08-22

### Fixed

- TorProxy controller watch services.

## [0.0.13] - 2023-08-21

### Added

- Helm test support.
- TorProxy Custom Resource Definition.
- TorProxy controller.

### Changed

- Upgrade dependencies.

## [0.0.12] - 2023-08-08

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

[Unreleased]: https://github.com/agabani/tor-operator/compare/v0.0.38...HEAD

[0.0.38]: https://github.com/agabani/tor-operator/compare/v0.0.37...v0.0.38
[0.0.37]: https://github.com/agabani/tor-operator/compare/v0.0.36...v0.0.37
[0.0.36]: https://github.com/agabani/tor-operator/compare/v0.0.35...v0.0.36
[0.0.35]: https://github.com/agabani/tor-operator/compare/v0.0.34...v0.0.35
[0.0.34]: https://github.com/agabani/tor-operator/compare/v0.0.33...v0.0.34
[0.0.33]: https://github.com/agabani/tor-operator/compare/v0.0.32...v0.0.33
[0.0.32]: https://github.com/agabani/tor-operator/compare/v0.0.31...v0.0.32
[0.0.31]: https://github.com/agabani/tor-operator/compare/v0.0.30...v0.0.31
[0.0.30]: https://github.com/agabani/tor-operator/compare/v0.0.29...v0.0.30
[0.0.29]: https://github.com/agabani/tor-operator/compare/v0.0.28...v0.0.29
[0.0.28]: https://github.com/agabani/tor-operator/compare/v0.0.27...v0.0.28
[0.0.27]: https://github.com/agabani/tor-operator/compare/v0.0.26...v0.0.27
[0.0.26]: https://github.com/agabani/tor-operator/compare/v0.0.25...v0.0.26
[0.0.25]: https://github.com/agabani/tor-operator/compare/v0.0.24...v0.0.25
[0.0.24]: https://github.com/agabani/tor-operator/compare/v0.0.23...v0.0.24
[0.0.23]: https://github.com/agabani/tor-operator/compare/v0.0.22...v0.0.23
[0.0.22]: https://github.com/agabani/tor-operator/compare/v0.0.21...v0.0.22
[0.0.21]: https://github.com/agabani/tor-operator/compare/v0.0.20...v0.0.21
[0.0.20]: https://github.com/agabani/tor-operator/compare/v0.0.19...v0.0.20
[0.0.19]: https://github.com/agabani/tor-operator/compare/v0.0.18...v0.0.19
[0.0.18]: https://github.com/agabani/tor-operator/compare/v0.0.17...v0.0.18
[0.0.17]: https://github.com/agabani/tor-operator/compare/v0.0.16...v0.0.17
[0.0.16]: https://github.com/agabani/tor-operator/compare/v0.0.15...v0.0.16
[0.0.15]: https://github.com/agabani/tor-operator/compare/v0.0.14...v0.0.15
[0.0.14]: https://github.com/agabani/tor-operator/compare/v0.0.13...v0.0.14
[0.0.13]: https://github.com/agabani/tor-operator/compare/v0.0.12...v0.0.13
[0.0.12]: https://github.com/agabani/tor-operator/compare/v0.0.11...v0.0.12
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
