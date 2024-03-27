# OnionService

An OnionService is an abstraction of a Tor Onion Service.

A Tor Onion Service is a service that can only be accessed over Tor.
Running a Tor Onion Service gives your users all the security of HTTPS with
the added privacy benefits of Tor.

## Screenshots

![OnionService](./onionservice.svg)

## Examples

### Minimal

The Tor Operator will create an OnionService using an auto generated OnionKey.

```
# onionkey.yaml
{% include "../../example/templates/onionservice_minimal/onionkey.yaml" %}
```

```
# onionservice.yaml
{% include "../../example/templates/onionservice_minimal/onionservice.yaml" %}
```

### Annotations, Labels and Names

The Tor Operator will create an OnionService using custom annotations, labels and names.

```
# onionkey.yaml
{% include "../../example/templates/onionservice_aln/onionkey.yaml" %}
```

```
# onionservice.yaml
{% include "../../example/templates/onionservice_aln/onionservice.yaml" %}
```

### Containers

The Tor Operator will partially configure existing containers and add additional containers to each Pod in the Deployment.

```
# onionkey.yaml
{% include "../../example/templates/onionservice_containers/onionkey.yaml" %}
```

```
# onionservice.yaml
{% include "../../example/templates/onionservice_containers/onionservice.yaml" %}
```

### Deployment

The Tor Operator will configure the Deployment.

```
# onionkey.yaml
{% include "../../example/templates/onionservice_deployment/onionkey.yaml" %}
```

```
# onionservice.yaml
{% include "../../example/templates/onionservice_deployment/onionservice.yaml" %}
```

### OnionBalance

The Tor Operator will create an OnionService registered with an OnionBalance using an auto generated OnionKey.

```
# onionkey.yaml
{% include "../../example/templates/onionservice_onionbalance/onionkey.yaml" %}
```

```
# onionservice.yaml
{% include "../../example/templates/onionservice_onionbalance/onionservice.yaml" %}
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
