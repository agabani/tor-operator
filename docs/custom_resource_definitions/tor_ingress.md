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

### Horizontal Pod Autoscaler

The Tor Operator will create a load balanced Onion Service using an auto generated Onion Key for the Onion Balance instance managed by a Horizontal Pod Autoscaler.

```
#onionkey.yaml
{% include "../../example/templates/toringress_hpa/onionkey.yaml" %}
```

```
#toringress.yaml
{% include "../../example/templates/toringress_hpa/toringress.yaml" %}
```

```
#hpa.yaml
{% include "../../example/templates/toringress_hpa/hpa.yaml" %}
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

## Features

### State

State can be observed in the status.

```
kubectl describe toringresses example
```

```
# ...
Status:
  State:     running
# ...
```

Possible values for `State`:

- `onion balance onion key not found`
- `onion balance onion key hostname not found`
- `onion service onion key hostname not found`
- `running`

## OpenAPI Spec

```
{% include "./toringress.yaml" %}
```