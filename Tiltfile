load('ext://namespace', 'namespace_create')

# =============================================================================
# Onionbalance
# =============================================================================
local_resource(
    'onionbalance: docker build',
    cmd='docker build -t agabani/onionbalance:dev ./containers/onionbalance',
    deps=['./containers/onionbalance/Dockerfile'],
    labels=['onionbalance'],
)

local_resource(
    'onionbalance: kind load',
    cmd='kind load docker-image agabani/onionbalance:dev',
    deps=['./containers/onionbalance/Dockerfile'],
    resource_deps=['onionbalance: docker build'],
    labels=['onionbalance'],
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

# =============================================================================
# Example
# =============================================================================
namespace_create('example')

k8s_yaml(helm(
    './example',
    name='example',
    namespace = 'example',
))
