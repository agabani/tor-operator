# TorIngress

A TorIngress is collection of OnionServices load balanced by a OnionBalance.

The user must provide the OnionKey for the OnionBalance.

The Tor Operator wil auto generate random OnionKeys for the OnionServices.

## Examples

### Basic

The Tor Operator will create a load balanced OnionService using an auto generated OnionKey for the OnionBalance instance.

```
#onionkey.yaml
{% include "../../example/templates/toringress/onionkey.yaml" %}
```

```
#toringress.yaml
{% include "../../example/templates/toringress/toringress.yaml" %}
```

### Horizontal Pod Autoscaler

The Tor Operator will create a load balanced OnionService using an auto generated OnionKey for the OnionBalance instance managed by a Horizontal Pod Autoscaler.

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

The Tor Operator will create a load balanced OnionService using an auto generated OnionKey for the OnionBalance instance.

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

- `OnionBalance OnionKey not found`
- `OnionBalance OnionKey hostname not found`
- `OnionService OnionKey hostname not found`
- `running`

## OpenAPI Spec

```
{% include "./toringress.yaml" %}
```
