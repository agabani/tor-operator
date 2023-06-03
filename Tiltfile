load('ext://namespace', 'namespace_create')

# =============================================================================
# Tor Operator
# =============================================================================
docker_build('agabani/tor-operator:dev', '.')

namespace_create('tor-operator')

k8s_yaml(helm(
    './helm',
    name='tor-operator',
    namespace = 'tor-operator',
    values='./Tiltfile.yaml'
))

# =============================================================================
# Example
# =============================================================================
namespace_create('example')

k8s_yaml(helm(
    './example',
    name='example',
    namespace = 'example'
))
