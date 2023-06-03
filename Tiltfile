load('ext://namespace', 'namespace_create')

namespace_create('tor-operator')

docker_build('agabani/tor-operator:dev', '.')

yaml = helm(
    './helm',
    name='tor-operator',
    namespace = 'tor-operator',
    values='./Tiltfile.yaml'
)

k8s_yaml(yaml)
