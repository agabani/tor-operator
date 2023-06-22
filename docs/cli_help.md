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

**Usage:** `tor-operator <COMMAND>`

###### **Subcommands:**

* `controller` — Controller
* `crd` — Custom Resource Definition
* `onion-key` — Onion Key



## `tor-operator controller`

Controller

**Usage:** `tor-operator controller <COMMAND>`

###### **Subcommands:**

* `run` — Run



## `tor-operator controller run`

Run

**Usage:** `tor-operator controller run [OPTIONS]`

###### **Options:**

* `--onion-balance-image-pull-policy <ONION_BALANCE_IMAGE_PULL_POLICY>`

  Default value: `IfNotPresent`
* `--onion-balance-image-uri <ONION_BALANCE_IMAGE_URI>`

  Default value: `ghcr.io/agabani/tor-operator:onion-balance-0.2.2`
* `--host <HOST>`

  Default value: `127.0.0.1`
* `--port <PORT>`

  Default value: `8080`
* `--tor-image-pull-policy <TOR_IMAGE_PULL_POLICY>`

  Default value: `IfNotPresent`
* `--tor-image-uri <TOR_IMAGE_URI>`

  Default value: `ghcr.io/agabani/tor-operator:tor-0.4.7.13`



## `tor-operator crd`

Custom Resource Definition

**Usage:** `tor-operator crd <COMMAND>`

###### **Subcommands:**

* `generate` — Generate



## `tor-operator crd generate`

Generate

**Usage:** `tor-operator crd generate [OPTIONS]`

###### **Options:**

* `--format <FORMAT>`

  Default value: `yaml`

  Possible values: `helm`, `json`, `yaml`

* `--output <OUTPUT>`



## `tor-operator onion-key`

Onion Key

**Usage:** `tor-operator onion-key <COMMAND>`

###### **Subcommands:**

* `generate` — Generate



## `tor-operator onion-key generate`

Generate

**Usage:** `tor-operator onion-key generate`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
