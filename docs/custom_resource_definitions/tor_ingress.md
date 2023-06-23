# Tor Ingress

A Tor Ingress is collection of Onion Services load balanced by a Onion Balance.

The user must provide the Onion Key for the Onion Balance.

The Tor Operator wil auto generate random Onion Keys for the Onion Services.

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
