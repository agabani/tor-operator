# Command-Line Help for `tor-operator`

This document contains the help content for the `tor-operator` command-line program.

**Command Overview:**

* [`tor-operator`↴](#tor-operator)
* [`tor-operator controller`↴](#tor-operator-controller)
* [`tor-operator controller run`↴](#tor-operator-controller-run)
* [`tor-operator crd`↴](#tor-operator-crd)
* [`tor-operator crd generate`↴](#tor-operator-crd-generate)
* [`tor-operator onion-key`↴](#tor-operator-onion-key)
* [`tor-operator onion-key generate`↴](#tor-operator-onion-key-generate)

## `tor-operator`

Tor Operator is a Kubernetes Operator that manages Onion Balances, Onion Keys and Onion Services to provide a highly available, load balanced and fault tolerate Tor Ingress and Tor Proxy.

**Usage:** `tor-operator [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `controller` — Controller
* `crd` — Custom Resource Definition
* `onion-key` — Onion Key

###### **Options:**

* `--otel-service-name <OTEL_SERVICE_NAME>` — Sets the value of the service.name resource attribute

  Default value: `tor-operator`
* `--otel-resource-attributes <OTEL_RESOURCE_ATTRIBUTES>` — Key-value pairs to be used as resource attributes
* `--otel-logs-exporter <OTEL_LOGS_EXPORTER>` — Specifies which exporters are used for logs

  Possible values: `console`, `otlp`

* `--otel-metrics-exporter <OTEL_METRICS_EXPORTER>` — Specifies which exporters are used for metrics

  Possible values: `console`, `otlp`

* `--otel-traces-exporter <OTEL_TRACES_EXPORTER>` — Specifies which exporters are used for traces

  Possible values: `console`, `otlp`

* `--otel-exporter-otlp-compression <OTEL_EXPORTER_OTLP_COMPRESSION>` — Specifies the OTLP transport compression to be used for all telemetry data

  Possible values: `gzip`, `zstd`

* `--otel-exporter-otlp-endpoint <OTEL_EXPORTER_OTLP_ENDPOINT>` — A base endpoint URL for any signal type, with an optionally-specified port number. Helpful for when you’re sending more than one signal to the same endpoint and want one environment variable to control the endpoint
* `--otel-exporter-otlp-headers <OTEL_EXPORTER_OTLP_HEADERS>` — A list of headers to apply to all outgoing data (traces, metrics, and logs)
* `--otel-exporter-otlp-protocol <OTEL_EXPORTER_OTLP_PROTOCOL>` — Specifies the OTLP transport protocol to be used for all telemetry data

  Possible values: `grpc`, `http/json`, `http/protobuf`

* `--otel-exporter-otlp-timeout <OTEL_EXPORTER_OTLP_TIMEOUT>` — The timeout value for all outgoing data (traces, metrics, and logs) in milliseconds

  Default value: `10000`
* `--otel-exporter-otlp-logs-compression <OTEL_EXPORTER_OTLP_LOGS_COMPRESSION>` — Specifies the OTLP transport compression to be used for log data

  Possible values: `gzip`, `zstd`

* `--otel-exporter-otlp-logs-endpoint <OTEL_EXPORTER_OTLP_LOGS_ENDPOINT>` — Endpoint URL for log data only, with an optionally-specified port number. Typically ends with `v1/logs` when using OTLP/HTTP
* `--otel-exporter-otlp-logs-headers <OTEL_EXPORTER_OTLP_LOGS_HEADERS>` — A list of headers to apply to all outgoing logs
* `--otel-exporter-otlp-logs-protocol <OTEL_EXPORTER_OTLP_LOGS_PROTOCOL>` — Specifies the OTLP transport protocol to be used for log data

  Possible values: `grpc`, `http/json`, `http/protobuf`

* `--otel-exporter-otlp-logs-timeout <OTEL_EXPORTER_OTLP_LOGS_TIMEOUT>` — The timeout value for all outgoing logs in milliseconds
* `--otel-exporter-otlp-metrics-compression <OTEL_EXPORTER_OTLP_METRICS_COMPRESSION>` — Specifies the OTLP transport compression to be used for metrics data

  Possible values: `gzip`, `zstd`

* `--otel-exporter-otlp-metrics-endpoint <OTEL_EXPORTER_OTLP_METRICS_ENDPOINT>` — Endpoint URL for metric data only, with an optionally-specified port number. Typically ends with `v1/metrics` when using OTLP/HTTP
* `--otel-exporter-otlp-metrics-headers <OTEL_EXPORTER_OTLP_METRICS_HEADERS>` — A list of headers to apply to all outgoing metrics
* `--otel-exporter-otlp-metrics-protocol <OTEL_EXPORTER_OTLP_METRICS_PROTOCOL>` — Specifies the OTLP transport protocol to be used for metrics data

  Possible values: `grpc`, `http/json`, `http/protobuf`

* `--otel-exporter-otlp-metrics-timeout <OTEL_EXPORTER_OTLP_METRICS_TIMEOUT>` — The timeout value for all outgoing metrics in milliseconds
* `--otel-exporter-otlp-traces-compression <OTEL_EXPORTER_OTLP_TRACES_COMPRESSION>` — Specifies the OTLP transport compression to be used for trace data

  Possible values: `gzip`, `zstd`

* `--otel-exporter-otlp-traces-endpoint <OTEL_EXPORTER_OTLP_TRACES_ENDPOINT>` — Endpoint URL for metric data only, with an optionally-specified port number. Typically ends with `v1/traces` when using OTLP/HTTP
* `--otel-exporter-otlp-traces-headers <OTEL_EXPORTER_OTLP_TRACES_HEADERS>` — A list of headers to apply to all outgoing traces
* `--otel-exporter-otlp-traces-protocol <OTEL_EXPORTER_OTLP_TRACES_PROTOCOL>` — Specifies the OTLP transport protocol to be used for trace data

  Possible values: `grpc`, `http/json`, `http/protobuf`

* `--otel-exporter-otlp-traces-timeout <OTEL_EXPORTER_OTLP_TRACES_TIMEOUT>` — The timeout value for all outgoing traces in milliseconds



## `tor-operator controller`

Controller

**Usage:** `tor-operator controller <COMMAND>`

###### **Subcommands:**

* `run` — Run the Tor Operator



## `tor-operator controller run`

Run the Tor Operator

**Usage:** `tor-operator controller run [OPTIONS]`

###### **Options:**

* `--onion-balance-image-pull-policy <ONION_BALANCE_IMAGE_PULL_POLICY>` — Onion Balance image pull policy

  Default value: `IfNotPresent`
* `--onion-balance-image-uri <ONION_BALANCE_IMAGE_URI>` — Onion Balance image uri

  Default value: `ghcr.io/agabani/tor-operator:onion-balance-0.2.3.0`
* `--host <HOST>` — Host the web server binds to

  Default value: `127.0.0.1`
* `--port <PORT>` — Port the web server binds to

  Default value: `8080`
* `--tor-image-pull-policy <TOR_IMAGE_PULL_POLICY>` — Tor image pull policy

  Default value: `IfNotPresent`
* `--tor-image-uri <TOR_IMAGE_URI>` — Tor image uri

  Default value: `ghcr.io/agabani/tor-operator:tor-0.4.8.16.0`



## `tor-operator crd`

Custom Resource Definition

**Usage:** `tor-operator crd <COMMAND>`

###### **Subcommands:**

* `generate` — Generate the Tor Operator CRDs



## `tor-operator crd generate`

Generate the Tor Operator CRDs

**Usage:** `tor-operator crd generate [OPTIONS]`

###### **Options:**

* `--format <FORMAT>` — Format of the CRDs

  Default value: `yaml`

  Possible values: `helm`, `json`, `yaml`

* `--output <OUTPUT>` — Output the CRDs into a directory



## `tor-operator onion-key`

Onion Key

**Usage:** `tor-operator onion-key <COMMAND>`

###### **Subcommands:**

* `generate` — Generate a random Tor Onion Key



## `tor-operator onion-key generate`

Generate a random Tor Onion Key

**Usage:** `tor-operator onion-key generate [OPTIONS]`

###### **Options:**

* `--output <OUTPUT>` — Output the Onion Keys into a directory



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
