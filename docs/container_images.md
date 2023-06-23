# Container Images

All container images are built for `linux/amd64` and `linux/arm64` platforms and can be found at [https://github.com/agabani/tor-operator/pkgs/container/tor-operator](https://github.com/agabani/tor-operator/pkgs/container/tor-operator)

## First Party

| Tor Operator                          | Type         |
| ------------------------------------- | ------------ |
| ghcr.io/agabani/tor-operator:\*.\*.\* | Release      |
| ghcr.io/agabani/tor-operator:main     | Experimental |

## Third Party

Tor Operator depends on third party images for Tor functionality.

The following images are provided as a convenience.

| Onion Balance                                       | Type    |
| --------------------------------------------------- | ------- |
| ghcr.io/agabani/tor-operator:onion-balance-\*.\*.\* | Release |

| Tor                                          | Type    |
| -------------------------------------------- | ------- |
| ghcr.io/agabani/tor-operator:tor-\*.\*.\*.\* | Release |
