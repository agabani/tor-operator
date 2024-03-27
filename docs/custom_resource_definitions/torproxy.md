# TorProxy

A TorProxy is collection of Tor clients load balanced by a Service.

## Screenshots

![TorProxy](./torproxy.svg)

## Examples

### Minimal

The Tor Operator will create a load balanced TorProxy instance.

```
# torproxy.yaml
{% include "../../example/templates/torproxy_minimal/torproxy.yaml" %}
```

### Annotations, Labels and Names

The Tor Operator will create an TorProxy using custom annotations, labels and names.

```
# torproxy.yaml
{% include "../../example/templates/torproxy_aln/torproxy.yaml" %}
```

### Containers

The Tor Operator will partially configure existing containers and add additional containers to each Pod in the Deployment.

```
# configmap.yaml
{% include "../../example/templates/torproxy_containers/configmap.yaml" %}
```

```
# torproxy.yaml
{% include "../../example/templates/torproxy_containers/torproxy.yaml" %}
```

### Deployment

The Tor Operator will configure the Deployment.

```
# torproxy.yaml
{% include "../../example/templates/torproxy_deployment/torproxy.yaml" %}
```

### HorizontalPodAutoscaler

The Tor Operator will create a load balanced TorProxy instance managed by a HorizontalPodAutoscaler.

```
# torproxy.yaml
{% include "../../example/templates/torproxy_hpa/torproxy.yaml" %}
```

### HorizontalPodAutoscaler (External)

The Tor Operator will create a load balanced TorProxy instance managed by an external HorizontalPodAutoscaler.

```
# torproxy.yaml
{% include "../../example/templates/torproxy_hpa_external/torproxy.yaml" %}
```

```
# hpa.yaml
{% include "../../example/templates/torproxy_hpa_external/hpa.yaml" %}
```

### Replica

The Tor Operator will create a load balanced TorProxy instance with a custom number of replicas.

```
# torproxy.yaml
{% include "../../example/templates/torproxy_replica/torproxy.yaml" %}
```

## Conditions

{%
  include "./torproxy.yaml"
  start="Represents the latest available observations of a deployment's current state."
  end="items:"
  dedent=true
%}

## OpenAPI Spec

```
{% include "./torproxy.yaml" %}
```
