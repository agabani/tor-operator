# TorIngress

A TorIngress is collection of OnionServices load balanced by a OnionBalance.

The user must provide the OnionKey for the OnionBalance.

The Tor Operator wil auto generate random OnionKeys for the OnionServices.

## Screenshots

![TorIngress](./toringress.svg)

## Examples

### Minimal

The Tor Operator will create a load balanced OnionService using an auto generated OnionKey for the OnionBalance instance.

```
# onionkey.yaml
{% include "../../example/templates/toringress_minimal/onionkey.yaml" %}
```

```
# toringress.yaml
{% include "../../example/templates/toringress_minimal/toringress.yaml" %}
```

### Annotations, Labels and Names

The Tor Operator will create an TorIngress using custom annotations, labels and names.

```
# onionkey.yaml
{% include "../../example/templates/toringress_aln/onionkey.yaml" %}
```

```
# toringress.yaml
{% include "../../example/templates/toringress_aln/toringress.yaml" %}
```

### Containers

The Tor Operator will partially configure existing containers and add additional containers to each Pod in the Deployment.

```
# configmap.yaml
{% include "../../example/templates/toringress_containers/configmap.yaml" %}
```

```
# onionkey.yaml
{% include "../../example/templates/toringress_containers/onionkey.yaml" %}
```

```
# toringress.yaml
{% include "../../example/templates/toringress_containers/toringress.yaml" %}
```

### Deployment

The Tor Operator will configure the Deployment.

```
# onionkey.yaml
{% include "../../example/templates/toringress_deployment/onionkey.yaml" %}
```

```
# toringress.yaml
{% include "../../example/templates/toringress_deployment/toringress.yaml" %}
```

### HorizontalPodAutoscaler

The Tor Operator will create a load balanced OnionService using an auto generated OnionKey for the OnionBalance instance managed by a HorizontalPodAutoscaler.

```
# onionkey.yaml
{% include "../../example/templates/toringress_hpa/onionkey.yaml" %}
```

```
# toringress.yaml
{% include "../../example/templates/toringress_hpa/toringress.yaml" %}
```

### HorizontalPodAutoscaler (External)

The Tor Operator will create a load balanced OnionService using an auto generated OnionKey for the OnionBalance instance managed by an external HorizontalPodAutoscaler.

```
# onionkey.yaml
{% include "../../example/templates/toringress_hpa_external/onionkey.yaml" %}
```

```
# toringress.yaml
{% include "../../example/templates/toringress_hpa_external/toringress.yaml" %}
```

```
# hpa.yaml
{% include "../../example/templates/toringress_hpa_external/hpa.yaml" %}
```

### Replica

The Tor Operator will create a load balanced OnionService using an auto generated OnionKey for the OnionBalance instance with a custom number of replicas.

```
# onionkey.yaml
{% include "../../example/templates/toringress_replica/onionkey.yaml" %}
```

```
# toringress.yaml
{% include "../../example/templates/toringress_replica/toringress.yaml" %}
```

### Torrc

The Tor Operator will prepend the template to the torrc file and substitute in the environment variables during container runtime.

```
# configmap.yaml
{% include "../../example/templates/toringress_torrc/configmap.yaml" %}
```

```
# onionkey.yaml
{% include "../../example/templates/toringress_torrc/onionkey.yaml" %}
```

```
# toringress.yaml
{% include "../../example/templates/toringress_torrc/toringress.yaml" %}
```

## Conditions

{%
  include "./toringress.yaml"
  start="Represents the latest available observations of a deployment's current state."
  end="items:"
  dedent=true
%}

## OpenAPI Spec

```
{% include "./toringress.yaml" %}
```
