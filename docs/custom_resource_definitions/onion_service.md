# Onion Service

An Onion Service is an abstraction of a Tor Onion Service.

A Tor Onion Service is a service that can only be accessed over Tor.
Running a Tor Onion Service gives your users all teh security of HTTPS with
the added privacy benefits of Tor.

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

### Full

The Tor Operator will create an Onion Service registered with an Onion Balance using an auto generated Onion Key.

```
#onionkey.yaml
{% include "../../example/templates/onionservice_full/onionkey.yaml" %}
```

```
#onionservice.yaml
{% include "../../example/templates/onionservice_full/onionservice.yaml" %}
```

## OpenAPI Spec

```
{% include "./onionservice.yaml" %}
```
