load('ext://namespace', 'namespace_create')

# =============================================================================
# Tor
# =============================================================================
local_resource(
    'tor: docker build',
    cmd='docker build -t agabani/tor:dev tor',
    deps=['./tor/Dockerfile'],
    labels=['tor'],
)

local_resource(
    'tor: kind load',
    cmd='kind load docker-image agabani/tor:dev',
    deps=['./tor/Dockerfile'],
    resource_deps =['tor: docker build'],
    labels=['tor'],
)

# =============================================================================
# Tor Operator
# =============================================================================
docker_build('agabani/tor-operator:dev', '.')

namespace_create('tor-operator')

k8s_yaml(helm(
    './helm',
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
