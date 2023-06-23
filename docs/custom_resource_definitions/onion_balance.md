# Onion Balance

An Onion Balance is an abstraction of a Tor Onion Balance.

Tor Onion Balance is the best way to load balance Tor Onion Services. The
load of introduction and rendezvous requests gets distributed across
multiple hosts while also increasing resiliency by eliminating single
points of failure.

## Examples

### Basic

The Tor Operator will create an Onion Balance using an auto generated Onion Key load load balancing a list of Onion Services.

```
#onionkey.yaml
{% include "../../example/templates/onionbalance/onionkey.yaml" %}
```

```
#onionbalance.yaml
{% include "../../example/templates/onionbalance/onionbalance.yaml" %}
```

### Full

The Tor Operator will create an Onion Balance using an auto generated Onion Key load load balancing a list of Onion Services.

```
#onionkey.yaml
{% include "../../example/templates/onionbalance_full/onionkey.yaml" %}
```

```
#onionbalance.yaml
{% include "../../example/templates/onionbalance_full/onionbalance.yaml" %}
```

## OpenAPI Spec

```
{% include "./onionbalance.yaml" %}
```
