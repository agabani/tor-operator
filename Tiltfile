docker_build('agabani/tor-operator:dev', '.')

k8s_yaml(helm('./helm', name='tor-operator', values='./Tiltfile.yaml'))
