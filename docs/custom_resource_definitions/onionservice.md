# OnionService

An OnionService is an abstraction of a Tor Onion Service.

A Tor Onion Service is a service that can only be accessed over Tor.
Running a Tor Onion Service gives your users all the security of HTTPS with
the added privacy benefits of Tor.

## Screenshots

![OnionService](./onionservice.svg)

## Examples

### Basic

The Tor Operator will create an OnionService using an auto generated OnionKey.

```
#onionkey.yaml
{% include "../../example/templates/onionservice/onionkey.yaml" %}
```

```
#onionservice.yaml
{% include "../../example/templates/onionservice/onionservice.yaml" %}
```

### OnionBalance

The Tor Operator will create an OnionService registered with an OnionBalance using an auto generated OnionKey.

```
#onionkey.yaml
{% include "../../example/templates/onionservice_onionbalance/onionkey.yaml" %}
```

```
#onionservice.yaml
{% include "../../example/templates/onionservice_onionbalance/onionservice.yaml" %}
```

### Full

The Tor Operator will create an OnionService registered with an OnionBalance using an auto generated OnionKey.

```
#onionkey.yaml
{% include "../../example/templates/onionservice_full/onionkey.yaml" %}
```

```
#onionservice.yaml
{% include "../../example/templates/onionservice_full/onionservice.yaml" %}
```

## Conditions

{%
  include "./onionservice.yaml"
  start="Represents the latest available observations of a deployment's current state."
  end="items:"
  dedent=true
%}

## OpenAPI Spec

```
{% include "./onionservice.yaml" %}
```
