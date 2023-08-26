load('ext://helm_resource', 'helm_repo', 'helm_resource')
load('ext://namespace', 'namespace_create')

# =============================================================================
# Kubernetes Dashboard
# =============================================================================
k8s_yaml('.kubernetes/kubernetes-dashboard/admin-user.yaml')
k8s_yaml('.kubernetes/kubernetes-dashboard/kubernetes-dashboard.yaml')

# =============================================================================
# Metrics Server
# =============================================================================
k8s_yaml('.kubernetes/metrics-server/metrics-server.yaml')

# =============================================================================
# Jaeger
# =============================================================================
helm_repo(
  'jaegertracing',
  'https://jaegertracing.github.io/helm-charts',
  labels=['jaeger'],
  resource_name='helm-repo-jaeger'
)

namespace_create('jaeger')

helm_resource(
  'jaeger',
  'jaegertracing/jaeger',
  flags=[
    '--set', 'provisionDataStore.cassandra=false',
    '--set', 'allInOne.enabled=true',
    '--set', 'storage.type=none',
    '--set', 'agent.enabled=false',
    '--set', 'collector.enabled=false',
    '--set', 'query.enabled=false',
    # '--set', 'hotrod.enabled=true',
    # '--set', 'hotrod.extraArgs[0]=--otel-exporter=otlp',
    # '--set', 'hotrod.extraEnv[0].name=OTEL_EXPORTER_OTLP_ENDPOINT',
    # '--set', 'hotrod.extraEnv[0].value=http://otel-collector-opentelemetry-collector.opentelemetry.svc:4318',
  ],
  labels=['jaeger'],
  namespace='jaeger',
  resource_deps=[
    'helm-repo-jaeger',
  ]
)

# =============================================================================
# Open Telemetry
# =============================================================================
helm_repo(
  'opentelemetry',
  'https://open-telemetry.github.io/opentelemetry-helm-charts',
  labels=['opentelemetry'],
  resource_name='helm-repo-opentelemetry'
)

namespace_create('opentelemetry')

helm_resource(
  'otel-collector',
  'open-telemetry/opentelemetry-collector',
  flags=[
    '--set', 'mode=deployment',
    '--set', 'presets.kubernetesAttributes.enabled=true',
    '--set', 'presets.kubeletMetrics.enabled=true',
    '--set', 'presets.logsCollection.enabled=true',
    '--set', 'config.exporters.jaeger.endpoint=jaeger-collector.jaeger.svc:14250',
    '--set', 'config.exporters.jaeger.tls.insecure=true',
    '--set', 'config.service.pipelines.traces.exporters[0]=logging',
    '--set', 'config.service.pipelines.traces.exporters[1]=jaeger',
  ],
  labels=['opentelemetry'],
  namespace='opentelemetry',
  resource_deps=[
    'helm-repo-opentelemetry',
  ]
)

helm_resource(
  'otel-collector-cluster',
  'open-telemetry/opentelemetry-collector',
  flags=[
    '--set', 'mode=deployment',
    '--set', 'replicaCount=1',
    '--set', 'presets.clusterMetrics.enabled=true',
    '--set', 'presets.kubernetesEvents.enabled=true',
    '--set', 'config.exporters.jaeger.endpoint=jaeger-collector.jaeger.svc:14250',
    '--set', 'config.exporters.jaeger.tls.insecure=true',
    '--set', 'config.service.pipelines.traces.exporters[0]=logging',
    '--set', 'config.service.pipelines.traces.exporters[1]=jaeger',
  ],
  labels=['opentelemetry'],
  namespace='opentelemetry',
  resource_deps=[
    'helm-repo-opentelemetry',
  ]
)

# =============================================================================
# Onion Balance
# =============================================================================
local_resource(
    'onion balance: docker build',
    cmd='docker build -t agabani/onion-balance:dev ./containers/onion-balance',
    deps=['./containers/onion-balance/Dockerfile'],
    labels=['onion-balance'],
)

local_resource(
    'onion balance: kind load',
    cmd='kind load docker-image agabani/onion-balance:dev',
    deps=['./containers/onion-balance/Dockerfile'],
    resource_deps=['onion balance: docker build'],
    labels=['onion-balance'],
)

# =============================================================================
# Tor
# =============================================================================
local_resource(
    'tor: docker build',
    cmd='docker build -t agabani/tor:dev ./containers/tor',
    deps=['./containers/tor/Dockerfile'],
    labels=['tor'],
)

local_resource(
    'tor: kind load',
    cmd='kind load docker-image agabani/tor:dev',
    deps=['./containers/tor/Dockerfile'],
    resource_deps=['tor: docker build'],
    labels=['tor'],
)

# =============================================================================
# Tor Operator
# =============================================================================
docker_build('agabani/tor-operator:dev', '.')

namespace_create('tor-operator')

k8s_yaml(helm(
    './charts/tor-operator',
    name='tor-operator',
    namespace = 'tor-operator',
    values='./Tiltfile.yaml',
))

k8s_resource('tor-operator', port_forwards=['8080:8080'])

# =============================================================================
# Example
# =============================================================================
namespace_create('example')

k8s_yaml(helm(
    './example',
    name='example',
    namespace = 'example',
))
