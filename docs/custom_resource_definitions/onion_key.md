# Onion Key

An Onion Key is an abstraction of a Tor Onion Key.

A Tor Onion Key consists of the following files:

- `hostname`
- `hs_ed25519_public_key`
- `hs_ed25519_public_key`

A user can import their existing Tor Onion keys by creating a secret.

```
 kubectl create secret generic tor-ingress-example \
   --from-file=hostname=./hostname \
   --from-file=hs_ed25519_public_key=./hs_ed25519_public_key \
   --from-file=hs_ed25519_secret_key=./hs_ed25519_secret_key
```

A user can have the Tor Operator create a new random Onion Key by using the
auto generate feature controlled by `.auto_generate`.

## Examples

### Imported

The Tor Operator will use the Onion Key provided in the `Secret`.

```
#secret.yaml
{% include "../../example/templates/onionkey/secret.yaml" %}
```

```
#onionkey.yaml
{% include "../../example/templates/onionkey/onionkey.yaml" %}
```

### Auto Generated

The Tor Operator will auto generated a random Onion Key and store it in a `Secret` on your behalf.

```
#onionkey.yaml
{% include "../../example/templates/onionkey_auto_generate/onionkey.yaml" %}
```

## OpenAPI Spec

```
{% include "./onionkey.yaml" %}
```
