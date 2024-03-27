# OnionBalance

An OnionBalance is an abstraction of a Tor Onion Balance.

Tor Onion Balance is the best way to load balance Tor Onion Services. The
load of introduction and rendezvous requests gets distributed across
multiple hosts while also increasing resiliency by eliminating single
points of failure.

## Screenshots

![OnionBalance](./onionbalance.svg)

## Examples

### Minimal

The Tor Operator will create an OnionBalance using an auto generated OnionKey load balancing a list of OnionServices.

```
# onionkey.yaml
{% include "../../example/templates/onionbalance_minimal/onionkey.yaml" %}
```

```
# onionbalance.yaml
{% include "../../example/templates/onionbalance_minimal/onionbalance.yaml" %}
```

### Annotations, Labels and Names

The Tor Operator will create an OnionBalance using custom annotations, labels and names.

```
# onionkey.yaml
{% include "../../example/templates/onionbalance_aln/onionkey.yaml" %}
```

```
# onionbalance.yaml
{% include "../../example/templates/onionbalance_aln/onionbalance.yaml" %}
```

### Containers

The Tor Operator will partially configure existing containers and add additional containers to each Pod in the Deployment.

```
# configmap.yaml
{% include "../../example/templates/onionbalance_containers/configmap.yaml" %}
```

```
# onionkey.yaml
{% include "../../example/templates/onionbalance_containers/onionkey.yaml" %}
```

```
# onionbalance.yaml
{% include "../../example/templates/onionbalance_containers/onionbalance.yaml" %}
```

### Deployment

The Tor Operator will configure the Deployment.

```
# onionkey.yaml
{% include "../../example/templates/onionbalance_deployment/onionkey.yaml" %}
```

```
# onionbalance.yaml
{% include "../../example/templates/onionbalance_deployment/onionbalance.yaml" %}
```

## Conditions

{%
  include "./onionbalance.yaml"
  start="Represents the latest available observations of a deployment's current state."
  end="items:"
  dedent=true
%}

## OpenAPI Spec

```
{% include "./onionbalance.yaml" %}
```
