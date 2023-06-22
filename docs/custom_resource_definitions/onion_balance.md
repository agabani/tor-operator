# Onion Balance

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

## OpenAPI Spec

```
{% include "./onionbalance.yaml" %}
```
