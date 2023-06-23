# Helm

Helm charts can be found at [https://agabani.github.io/tor-operator/index.yaml](https://agabani.github.io/tor-operator/index.yaml)

## Installation

1.  Add the chart repository.

        helm repo add agabani-tor-operator https://agabani.github.io/tor-operator

2.  Install the Tor Operator.

        helm install tor-operator agabani-tor-operator/tor-operator \
            --create-namespace \
            --namespace tor-operator \
            --set image.tag=main

## Values

```
{% include "../charts/tor-operator/values.yaml" %}
```
