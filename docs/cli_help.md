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

* `--opentelemetry-endpoint <OPENTELEMETRY_ENDPOINT>` — OpenTelemetry endpoint



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

  Default value: `ghcr.io/agabani/tor-operator:onion-balance-0.2.2.1`
* `--host <HOST>` — Host the web server binds to

  Default value: `127.0.0.1`
* `--port <PORT>` — Port the web server binds to

  Default value: `8080`
* `--tor-image-pull-policy <TOR_IMAGE_PULL_POLICY>` — Tor image pull policy

  Default value: `IfNotPresent`
* `--tor-image-uri <TOR_IMAGE_URI>` — Tor image uri

  Default value: `ghcr.io/agabani/tor-operator:tor-0.4.8.13.0`



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
