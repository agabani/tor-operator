# TorProxy

A TorProxy is collection of Tor clients load balanced by a Service.

## Screenshots

![TorProxy](./torproxy.svg)

## Examples

### Basic

The Tor Operator will create a load balanced TorProxy instance.

```
#torproxy.yaml
{% include "../../example/templates/torproxy/torproxy.yaml" %}
```

### HorizontalPodAutoscaler

The Tor Operator will create a load balanced TorProxy instance managed by a HorizontalPodAutoscaler.

```
#torproxy.yaml
{% include "../../example/templates/torproxy_hpa/torproxy.yaml" %}
```

### External HorizontalPodAutoscaler

The Tor Operator will create a load balanced TorProxy instance managed by an external HorizontalPodAutoscaler.

```
#torproxy.yaml
{% include "../../example/templates/torproxy_hpa_external/torproxy.yaml" %}
```

```
#hpa.yaml
{% include "../../example/templates/torproxy_hpa_external/hpa.yaml" %}
```

### Full

The Tor Operator will create a load balanced TorProxy instance managed by a HorizontalPodAutoscaler.

```
#torproxy.yaml
{% include "../../example/templates/torproxy_full/torproxy.yaml" %}
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
