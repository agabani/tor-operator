load('ext://helm_resource', 'helm_repo', 'helm_resource')
load('ext://namespace', 'namespace_create')

# =============================================================================
# HyperDX
# =============================================================================
namespace_create('hyperdx')

k8s_yaml('.kubernetes/hyperdx/hyperdx.yaml')

k8s_resource(
  'hyperdx',
  port_forwards=[
    '8000:8000',
    '8080:8080',
  ],
  labels=['hyperdx']
)

# =============================================================================
# Metrics Server
# =============================================================================
k8s_yaml('.kubernetes/metrics-server/metrics-server.yaml')

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

k8s_resource('tor-operator', port_forwards=['8888:8080'])

# =============================================================================
# Example
# =============================================================================
namespace_create('example')

k8s_yaml(helm(
    './example',
    name='example',
    namespace = 'example',
))
