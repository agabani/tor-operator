# Tor Operator

## Problem Statement

I would like to access my Raspberry Pi Kubernetes Cluster from the internet without needing to open up ports on my router.

## Proposed solution

Build a kubernetes operator to expose kubernetes services as Hidden Services on the Tor Network.

Rationale: Free. No extra servers to maintain.

## Alternative solution

Expose kubernetes services using HTTPS Tunnels to a hosted server.

Cons: Paid. Extra servers to maintain.

## Learning Opportunities

* Helm Kata
* Kubernetes Operator Kata
* Rust Kata
  * AMD64/ARM64
  * Unix/Windows
* Tor Network Kata
