use std::{fs::File, io::Write};

use tor_operator::{
    cli::{
        parse, CliArgs, CliCommands, ControllerArgs, ControllerCommands, ControllerRunArgs,
        CrdArgs, CrdCommands, CrdGenerateArgs, CrdGenerateArgsFormat, OnionKeyArgs,
        OnionKeyCommands, OnionKeyGenerateArgs,
    },
    crypto, http_server, onion_balance, onion_key, onion_service,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = &parse();

    match &cli.command {
        CliCommands::Controller(controller) => match &controller.command {
            ControllerCommands::Run(run) => controller_run(cli, controller, run).await,
        },
        CliCommands::Crd(crd) => match &crd.command {
            CrdCommands::Generate(generate) => crd_generate(cli, crd, generate),
        },
        CliCommands::OnionKey(onion_address) => match &onion_address.command {
            OnionKeyCommands::Generate(generate) => {
                onion_key_generate(cli, onion_address, generate)
            }
        },
    }
}

async fn controller_run(_cli: &CliArgs, _controller: &ControllerArgs, run: &ControllerRunArgs) {
    let addr = format!("{}:{}", run.host, run.port).parse().unwrap();

    let onion_balance_config = onion_balance::Config {
        onion_balance_image: onion_balance::ImageConfig {
            pull_policy: run.onion_balance_image_pull_policy.clone(),
            uri: run.onion_balance_image_uri.clone(),
        },
        tor_image: onion_balance::ImageConfig {
            pull_policy: run.tor_image_pull_policy.clone(),
            uri: run.tor_image_uri.clone(),
        },
    };

    let onion_key_config = onion_key::Config {};

    let onion_service_config = onion_service::Config {
        tor_image: onion_service::ImageConfig {
            pull_policy: run.tor_image_pull_policy.clone(),
            uri: run.tor_image_uri.clone(),
        },
    };

    tokio::select! {
        _ = http_server::run(addr) => {},
        _ = onion_balance::run_controller(onion_balance_config) => {},
        _ = onion_key::run_controller(onion_key_config) => {},
        _ = onion_service::run_controller(onion_service_config) => {},
    }
}

fn crd_generate(_cli: &CliArgs, _crd: &CrdArgs, generate: &CrdGenerateArgs) {
    fn helmify(content: String) -> String {
        format!(
            "{}\n{}{}\n",
            "{{- if .Values.customResourceDefinition.create -}}",
            content.replace(
                "\nspec:",
                &vec![
                    "",
                    "  labels:",
                    "    {{- include \"tor-operator.labels\" . | nindent 4 }}",
                    "  {{- with .Values.customResourceDefinition.annotations }}",
                    "  annotations:",
                    "    {{- toYaml . | nindent 4 }}",
                    "  {{- end }}",
                    "spec:",
                ]
                .join("\n"),
            ),
            "{{- end }}"
        )
    }

    let crds = vec![
        (
            "onionbalance",
            onion_balance::generate_custom_resource_definition(),
        ),
        (
            //
            "onionkey",
            onion_key::generate_custom_resource_definition(),
        ),
        (
            "onionservice",
            onion_service::generate_custom_resource_definition(),
        ),
    ];

    for (name, crd) in crds {
        let content = match generate.format {
            CrdGenerateArgsFormat::Helm => helmify(serde_yaml::to_string(&crd).unwrap()),
            CrdGenerateArgsFormat::Json => serde_json::to_string_pretty(&crd).unwrap(),
            CrdGenerateArgsFormat::Yaml => serde_yaml::to_string(&crd).unwrap(),
        };

        if let Some(output) = &generate.output {
            let path = match generate.format {
                CrdGenerateArgsFormat::Helm => output.join(format!("{name}.yaml")),
                CrdGenerateArgsFormat::Json => output.join(format!("{name}.json")),
                CrdGenerateArgsFormat::Yaml => output.join(format!("{name}.yaml")),
            };

            File::create(path)
                .unwrap()
                .write_all(content.as_bytes())
                .unwrap();
        } else {
            print!("{content}");
        }
    }
}

fn onion_key_generate(_cli: &CliArgs, _onion_key: &OnionKeyArgs, _generate: &OnionKeyGenerateArgs) {
    let expanded_secret_key = crypto::ExpandedSecretKey::generate();
    let public_key = expanded_secret_key.public_key();

    let hostname = public_key.hostname();
    let hidden_service_public_key = crypto::HiddenServicePublicKey::from_public_key(&public_key);
    let hidden_service_secret_key =
        crypto::HiddenServiceSecretKey::from_expanded_secret_key(&expanded_secret_key);

    File::create("hostname")
        .unwrap()
        .write_all(hostname.as_bytes())
        .unwrap();

    File::create("hs_ed25519_public_key")
        .unwrap()
        .write_all(&hidden_service_public_key.to_bytes())
        .unwrap();

    File::create("hs_ed25519_secret_key")
        .unwrap()
        .write_all(&hidden_service_secret_key.to_bytes())
        .unwrap();
}
