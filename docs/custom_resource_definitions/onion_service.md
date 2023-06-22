# Onion Service

## Examples

### Basic

The Tor Operator will create an Onion Service using an auto generated Onion Key.

```
#onionkey.yaml
{% include "../../example/templates/onionservice/onionkey.yaml" %}
```

```
#onionservice.yaml
{% include "../../example/templates/onionservice/onionservice.yaml" %}
```

### Onion Balance

The Tor Operator will create an Onion Service registered with an Onion Balance using an auto generated Onion Key.

```
#onionkey.yaml
{% include "../../example/templates/onionservice_onionbalance/onionkey.yaml" %}
```

```
#onionservice.yaml
{% include "../../example/templates/onionservice_onionbalance/onionservice.yaml" %}
```

## OpenAPI Spec

```
{% include "./onionservice.yaml" %}
```
