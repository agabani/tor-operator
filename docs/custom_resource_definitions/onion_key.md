# Onion Key

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
