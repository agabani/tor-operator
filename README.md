# Tor Operator

Tor Operator is a Kubernetes Operator that manages [Onion Balances](https://agabani.github.io/tor-operator/docs/custom_resource_definitions/onionbalance/), [Onion Keys](https://agabani.github.io/tor-operator/docs/custom_resource_definitions/onionkey/) and [Onion Services](https://agabani.github.io/tor-operator/docs/custom_resource_definitions/onionservice/) to provide a highly available, load balanced and fault tolerate [Tor Ingress](https://agabani.github.io/tor-operator/docs/custom_resource_definitions/toringress/) and [Tor Proxy](https://agabani.github.io/tor-operator/docs/custom_resource_definitions/torproxy/).

## Documentation

[https://agabani.github.io/tor-operator/docs/](https://agabani.github.io/tor-operator/docs/)

<!--getting-started-installation-start-->

## Installation

1.  Add the chart repository.

        helm repo add agabani-tor-operator https://agabani.github.io/tor-operator

1.  Update the chart repository.

        helm repo update agabani-tor-operator

1.  Install the Tor Operator.

        helm upgrade tor-operator agabani-tor-operator/tor-operator \
            --create-namespace \
            --install \
            --namespace tor-operator

1.  Test the Tor Operator.

        helm test tor-operator --namespace tor-operator

<!--getting-started-installation-end-->

<!--getting-started-custom-resource-definitions-start-->

## Creating a Tor Ingress

1.  Prepare your existing Onion Key to look like:

    - `hostname`
    - `hs_ed25519_public_key`
    - `hs_ed25519_secret_key`

    or generate a new Onion Key using:

        cargo install --git https://github.com/agabani/tor-operator --tag v0.0.37
        tor-operator onion-key generate

1.  Create a `Secret` containing the Onion Key.

        kubectl create secret generic tor-ingress-example \
            --from-file=hostname=./hostname \
            --from-file=hs_ed25519_public_key=./hs_ed25519_public_key \
            --from-file=hs_ed25519_secret_key=./hs_ed25519_secret_key

1.  Create an `OnionKey` wrapping the `Secret`.

        # onionkey.yaml
        apiVersion: tor.agabani.co.uk/v1
        kind: OnionKey
        metadata:
          name: tor-ingress-example
        spec:
          secret:
            name: tor-ingress-example

    `kubectl apply -f onionkey.yaml`

1.  Create a `TorIngress`, changing `example:80` to your targets `host:port`

        # toringress.yaml
        apiVersion: tor.agabani.co.uk/v1
        kind: TorIngress
        metadata:
          name: tor-ingress-example
        spec:
          horizontalPodAutoscaler:
            maxReplicas: 6
            minReplicas: 3
          onionBalance:
            onionKey:
              name: tor-ingress-example
          onionService:
            deployment:
              containers:
                - name: tor
                  resources:
                    requests:
                      cpu: 100m
            ports:
              - target: example:80
                virtport: 80

    `kubectl apply -f toringress.yaml`

## Creating a Tor Proxy

1.  Create a `TorProxy`

        # torproxy.yaml
        apiVersion: tor.agabani.co.uk/v1
        kind: TorProxy
        metadata:
          name: tor-proxy-example
        spec:
          deployment:
            containers:
              - name: tor
                resources:
                  requests:
                    cpu: 100m
          horizontalPodAutoscaler:
            maxReplicas: 4
            minReplicas: 2
          service:
            ports:
              - name: http-tunnel
                port: 1080
                protocol: HTTP_TUNNEL
              - name: socks
                port: 9050
                protocol: SOCKS

    `kubectl apply -f torproxy.yaml`

<!--getting-started-custom-resource-definitions-end-->

## Screenshots

![OnionBalance](./docs/custom_resource_definitions/onionbalance.svg)

![OnionKey](./docs/custom_resource_definitions/onionkey.svg)

![OnionService](./docs/custom_resource_definitions/onionservice.svg)

![TorIngress](./docs/custom_resource_definitions/toringress.svg)

![TorProxy](./docs/custom_resource_definitions/torproxy.svg)
