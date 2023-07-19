# Metrics

Metrics are accessible through the `[GET] /metrics` HTTP endpoint.

## Examples

```
# TYPE tor_operator_kubernetes_api_usage_total counter
tor_operator_kubernetes_api_usage_total{kind="Deployment",group="apps",verb="watch",version="v1"} 1
tor_operator_kubernetes_api_usage_total{kind="OnionKey",group="tor.agabani.co.uk",verb="watch",version="v1"} 1
tor_operator_kubernetes_api_usage_total{kind="OnionBalance",group="tor.agabani.co.uk",verb="patch",version="v1"} 22
tor_operator_kubernetes_api_usage_total{kind="OnionService",group="tor.agabani.co.uk",verb="list",version="v1"} 135
tor_operator_kubernetes_api_usage_total{kind="Secret",group="",verb="list",version="v1"} 60
tor_operator_kubernetes_api_usage_total{kind="Deployment",group="apps",verb="list",version="v1"} 126
tor_operator_kubernetes_api_usage_total{kind="Secret",group="",verb="watch",version="v1"} 1
tor_operator_kubernetes_api_usage_total{kind="OnionBalance",group="tor.agabani.co.uk",verb="watch",version="v1"} 1
tor_operator_kubernetes_api_usage_total{kind="OnionKey",group="tor.agabani.co.uk",verb="get",version="v1"} 277
tor_operator_kubernetes_api_usage_total{kind="OnionBalance",group="tor.agabani.co.uk",verb="list",version="v1"} 135
tor_operator_kubernetes_api_usage_total{kind="Secret",group="",verb="get",version="v1"} 62
tor_operator_kubernetes_api_usage_total{kind="OnionKey",group="tor.agabani.co.uk",verb="list",version="v1"} 139
tor_operator_kubernetes_api_usage_total{kind="ConfigMap",group="",verb="list",version="v1"} 126
tor_operator_kubernetes_api_usage_total{kind="ConfigMap",group="",verb="patch",version="v1"} 24
tor_operator_kubernetes_api_usage_total{kind="OnionService",group="tor.agabani.co.uk",verb="delete",version="v1"} 3
tor_operator_kubernetes_api_usage_total{kind="HorizontalPodAutoscaler",group="autoscaling",verb="patch",version="v2"} 126
tor_operator_kubernetes_api_usage_total{kind="TorIngress",group="tor.agabani.co.uk",verb="patch",version="v1"} 18
tor_operator_kubernetes_api_usage_total{kind="OnionService",group="tor.agabani.co.uk",verb="patch",version="v1"} 33
tor_operator_kubernetes_api_usage_total{kind="OnionKey",group="tor.agabani.co.uk",verb="delete",version="v1"} 3
tor_operator_kubernetes_api_usage_total{kind="ConfigMap",group="",verb="watch",version="v1"} 1
tor_operator_kubernetes_api_usage_total{kind="OnionKey",group="tor.agabani.co.uk",verb="patch",version="v1"} 50
tor_operator_kubernetes_api_usage_total{kind="HorizontalPodAutoscaler",group="autoscaling",verb="list",version="v2"} 135
tor_operator_kubernetes_api_usage_total{kind="Deployment",group="apps",verb="patch",version="v1"} 126
tor_operator_kubernetes_api_usage_total{kind="Secret",group="",verb="patch",version="v1"} 23

# TYPE tor_operator_reconciliations_total counter
tor_operator_reconciliations_total{controller="tor-ingress"} 145
tor_operator_reconciliations_total{controller="onion-service"} 77
tor_operator_reconciliations_total{controller="onion-key"} 62
tor_operator_reconciliations_total{controller="onion-balance"} 55

# TYPE tor_operator_reconcile_duration_seconds histogram
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="0.01"} 4
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="0.1"} 141
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="0.25"} 145
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="0.5"} 145
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="1"} 145
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="5"} 145
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="15"} 145
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="60"} 145
tor_operator_reconcile_duration_seconds_bucket{controller="tor-ingress",le="+Inf"} 145
tor_operator_reconcile_duration_seconds_sum{controller="tor-ingress"} 4.531024465000001
tor_operator_reconcile_duration_seconds_count{controller="tor-ingress"} 145
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="0.01"} 12
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="0.1"} 62
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="0.25"} 62
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="0.5"} 62
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="1"} 62
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="5"} 62
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="15"} 62
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="60"} 62
tor_operator_reconcile_duration_seconds_bucket{controller="onion-key",le="+Inf"} 62
tor_operator_reconcile_duration_seconds_sum{controller="onion-key"} 1.3781806049999994
tor_operator_reconcile_duration_seconds_count{controller="onion-key"} 62
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="0.01"} 3
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="0.1"} 77
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="0.25"} 77
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="0.5"} 77
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="1"} 77
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="5"} 77
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="15"} 77
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="60"} 77
tor_operator_reconcile_duration_seconds_bucket{controller="onion-service",le="+Inf"} 77
tor_operator_reconcile_duration_seconds_sum{controller="onion-service"} 2.134455453
tor_operator_reconcile_duration_seconds_count{controller="onion-service"} 77
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="0.01"} 3
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="0.1"} 55
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="0.25"} 55
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="0.5"} 55
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="1"} 55
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="5"} 55
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="15"} 55
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="60"} 55
tor_operator_reconcile_duration_seconds_bucket{controller="onion-balance",le="+Inf"} 55
tor_operator_reconcile_duration_seconds_sum{controller="onion-balance"} 1.3091172410000005
tor_operator_reconcile_duration_seconds_count{controller="onion-balance"} 55
```
