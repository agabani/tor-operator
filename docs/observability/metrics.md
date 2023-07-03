# Metrics

Metrics are accessible through the `[GET] /metrics` HTTP endpoint.

## Examples

```
# TYPE tor_operator_reconciliations_total counter
tor_operator_reconciliations_total{controller="onion-service"} 39
tor_operator_reconciliations_total{controller="onion-balance"} 8
tor_operator_reconciliations_total{controller="onion-key"} 15
tor_operator_reconciliations_total{controller="tor-ingress"} 8

# TYPE tor_operator_kubernetes_api_usage_total counter
tor_operator_kubernetes_api_usage_total{kind="OnionBalance",group="tor.agabani.co.uk",verb="watch",version="v1"} 1
tor_operator_kubernetes_api_usage_total{kind="Secret",group="",verb="watch",version="v1"} 1
tor_operator_kubernetes_api_usage_total{kind="OnionService",group="tor.agabani.co.uk",verb="watch",version="v1"} 1
tor_operator_kubernetes_api_usage_total{kind="Deployment",group="apps",verb="list",version="v1"} 47
tor_operator_kubernetes_api_usage_total{kind="Secret",group="",verb="get",version="v1"} 15
tor_operator_kubernetes_api_usage_total{kind="ConfigMap",group="",verb="watch",version="v1"} 2
tor_operator_kubernetes_api_usage_total{kind="OnionKey",group="tor.agabani.co.uk",verb="get",version="v1"} 55
tor_operator_kubernetes_api_usage_total{kind="Secret",group="",verb="list",version="v1"} 14
tor_operator_kubernetes_api_usage_total{kind="Deployment",group="apps",verb="watch",version="v1"} 2
tor_operator_kubernetes_api_usage_total{kind="ConfigMap",group="",verb="list",version="v1"} 47
tor_operator_kubernetes_api_usage_total{kind="Deployment",group="apps",verb="delete",version="v1"} 6
tor_operator_kubernetes_api_usage_total{kind="OnionBalance",group="tor.agabani.co.uk",verb="list",version="v1"} 8
tor_operator_kubernetes_api_usage_total{kind="Deployment",group="apps",verb="patch",version="v1"} 47
tor_operator_kubernetes_api_usage_total{kind="OnionKey",group="tor.agabani.co.uk",verb="list",version="v1"} 8
tor_operator_kubernetes_api_usage_total{kind="OnionService",group="tor.agabani.co.uk",verb="list",version="v1"} 8
tor_operator_kubernetes_api_usage_total{kind="OnionService",group="tor.agabani.co.uk",verb="patch",version="v1"} 6
tor_operator_kubernetes_api_usage_total{kind="OnionKey",group="tor.agabani.co.uk",verb="watch",version="v1"} 1

# TYPE tor_operator_reconcile_duration_seconds histogram
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="0.01"} 0
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="0.1"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="0.25"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="0.5"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="1"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="5"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="15"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="60"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="+Inf"} 8
tor_operator_reconcile_duration_seconds_sum{controller="tor-ingress"} 0.39888715900000005
tor_operator_reconcile_duration_seconds_count{controller="tor-ingress"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="0.01"} 1
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="0.1"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="0.25"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="0.5"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="1"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="5"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="15"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="60"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="+Inf"} 8
tor_operator_reconcile_duration_seconds_sum{controller="onion-balance"} 0.294052989
tor_operator_reconcile_duration_seconds_count{controller="onion-balance"} 8
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="0.01"} 3
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="0.1"} 39
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="0.25"} 39
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="0.5"} 39
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="1"} 39
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="5"} 39
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="15"} 39
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="60"} 39
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="+Inf"} 39
tor_operator_reconcile_duration_seconds_sum{controller="onion-service"} 1.2481789289999998
tor_operator_reconcile_duration_seconds_count{controller="onion-service"} 39
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="0.01"} 2
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="0.1"} 15
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="0.25"} 15
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="0.5"} 15
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="1"} 15
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="5"} 15
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="15"} 15
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="60"} 15
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="+Inf"} 15
tor_operator_reconcile_duration_seconds_sum{controller="onion-key"} 0.769586808
tor_operator_reconcile_duration_seconds_count{controller="onion-key"} 15
```
