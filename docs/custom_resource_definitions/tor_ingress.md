# Tor Ingress

## Examples

### Basic

The Tor Operator will create a load balanced Onion Service using an auto generated Onion Key for the Onion Balance instance.

```
#onionkey.yaml
{% include "../../example/templates/toringress/onionkey.yaml" %}
```

```
#toringress.yaml
{% include "../../example/templates/toringress/toringress.yaml" %}
```

### Full

The Tor Operator will create a load balanced Onion Service using an auto generated Onion Key for the Onion Balance instance.

```
#onionkey.yaml
{% include "../../example/templates/toringress_full/onionkey.yaml" %}
```

```
#toringress.yaml
{% include "../../example/templates/toringress_full/toringress.yaml" %}
```

## OpenAPI Spec

```
{% include "./toringress.yaml" %}
```
