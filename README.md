# Tor Operator

## Installation

```terminal
helm repo add agabani-tor-operator https://agabani.github.io/tor-operator

helm install tor-operator agabani-tor-operator/tor-operator --create-namespace --namespace tor-operator --set image.repository=ghcr.io/agabani/tor-operator --set image.tag=main --set tor.image.repository=ghcr.io/agabani/tor-operator --set tor.image.tag=tor-0.4.7.13
```

## Creating a Tor Onion Service

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: tor-onion-service-example
data:
  hostname: ...
  hs_ed25519_public_key: ...
  hs_ed25519_secret_key: ...
```

```yaml
apiVersion: tor.agabani.co.uk/v1
kind: OnionService
metadata:
  name: tor-onion-service-example
spec:
  hidden_service_ports:
    - target: example:80
      virtport: 80
  secret_name: tor-onion-service-example
```

## Tutorial

1.  Install the Tor Operator

    ```terminal
    helm repo add agabani-tor-operator https://agabani.github.io/tor-operator

    helm install tor-operator agabani-tor-operator/tor-operator --create-namespace --namespace tor-operator --set image.repository=ghcr.io/agabani/tor-operator --set image.tag=main --set tor.image.repository=ghcr.io/agabani/tor-operator --set tor.image.tag=tor-0.4.7.13
    ```

1.  Install a web server

    ```
    helm install example oci://registry-1.docker.io/bitnamicharts/nginx --create-namespace --namespace example --set service.type=ClusterIP
    ```

1.  Create a secret containing Tor Onion Service hidden service files

    ```
    kubectl -n example create secret generic tor-onion-service-example-nginx --from-file=hostname=./hostname --from-file=hs_ed25519_public_key=./hs_ed25519_public_key --from-file=hs_ed25519_secret_key=./hs_ed25519_secret_key
    ```

1.  Create a `onionservice.yaml` with contents:

    ```yaml
    apiVersion: tor.agabani.co.uk/v1
    kind: OnionService
    metadata:
      name: example-nginx
    spec:
      hidden_service_ports:
        - target: example-nginx:80
          virtport: 80
      secret_name: tor-onion-service-example-nginx
    ```

1.  Create the Tor Onion Service

    ```
    kubectl -n example apply -f onionservice.yaml
    ```

1.  Visit the `*****.onion` address using your Tor Browser
