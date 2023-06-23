# Tor Operator

Tor Operator is a Kubernetes Operator that manages [Onion Balances](./custom_resource_definitions/onion_balance.md), [Onion Keys](./custom_resource_definitions/onion_key.md) and [Onion Services](./custom_resource_definitions/onion_service.md) to provide a highly available, load balanced and fault tolerate [Tor Ingress](./custom_resource_definitions/tor_ingress.md).

## Installation

1.  Add the chart repository.

        helm repo add agabani-tor-operator https://agabani.github.io/tor-operator

1.  Install the Tor Operator.

        helm install tor-operator agabani-tor-operator/tor-operator \
            --create-namespace \
            --namespace tor-operator \
            --set image.tag=main

## Creating a Tor Ingress

1.  Prepare your existing Onion Key to look like:

    - `hostname`
    - `hs_ed25519_public_key`
    - `hs_ed25519_public_key`

    or generate a new Onion Key using:

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
          onion_balance:
            onion_key:
              name: tor-ingress-example
          onion_service:
            ports:
              - target: example:80
                virtport: 80
            replicas: 3

    `kubectl apply -f toringress.yaml`

## Documentation

[https://agabani.github.io/tor-operator/docs/](https://agabani.github.io/tor-operator/docs/)
