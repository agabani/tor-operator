use std::{borrow::Cow, fs::File, io::Write};

use kube::Client;
use tor_operator::{
    cli::{
        parse, CliArgs, CliCommands, ControllerArgs, ControllerCommands, ControllerRunArgs,
        CrdArgs, CrdCommands, CrdGenerateArgs, CrdGenerateArgsFormat, MarkdownArgs,
        MarkdownCommands, MarkdownGenerateArgs, OnionKeyArgs, OnionKeyCommands,
        OnionKeyGenerateArgs,
    },
    http_server,
    metrics::Metrics,
    onion_balance, onion_key, onion_service,
    tor::{ExpandedSecretKey, HiddenServicePublicKey, HiddenServiceSecretKey, Hostname, PublicKey},
    tor_ingress, tor_proxy,
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
        CliCommands::Markdown(markdown) => match &markdown.command {
            MarkdownCommands::Generate(help) => markdown_generate(cli, markdown, help),
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

    let client = Client::try_default().await.unwrap();

    let metrics = Metrics::new();

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

    let tor_ingress_config = tor_ingress::Config {};

    let tor_proxy_config = tor_proxy::Config {
        tor_image: tor_proxy::ImageConfig {
            pull_policy: run.tor_image_pull_policy.clone(),
            uri: run.tor_image_uri.clone(),
        },
    };

    tokio::select! {
        _ = http_server::run(addr, metrics.clone()) => {},
        _ = onion_balance::run_controller(client.clone(), onion_balance_config, metrics.clone()) => {},
        _ = onion_key::run_controller(client.clone(), onion_key_config, metrics.clone()) => {},
        _ = onion_service::run_controller(client.clone(),onion_service_config, metrics.clone()) => {},
        _ = tor_ingress::run_controller(client.clone(), tor_ingress_config, metrics.clone()) => {},
        _ = tor_proxy::run_controller(client.clone(), tor_proxy_config, metrics.clone()) => {},
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
        (
            "toringress",
            tor_ingress::generate_custom_resource_definition(),
        ),
        (
            //
            "torproxy",
            tor_proxy::generate_custom_resource_definition(),
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

fn markdown_generate(_cli: &CliArgs, _markdown: &MarkdownArgs, generate: &MarkdownGenerateArgs) {
    if let Some(output) = &generate.output {
        File::create(output)
            .unwrap()
            .write_all(clap_markdown::help_markdown::<CliArgs>().as_bytes())
            .unwrap();
    } else {
        clap_markdown::print_help_markdown::<CliArgs>();
    }
}

fn onion_key_generate(_cli: &CliArgs, _onion_key: &OnionKeyArgs, generate: &OnionKeyGenerateArgs) {
    let expanded_secret_key = ExpandedSecretKey::generate();
    let public_key = PublicKey::from(&expanded_secret_key);

    let hostname = Hostname::from(&public_key);
    let hidden_service_public_key = HiddenServicePublicKey::from(&public_key);
    let hidden_service_secret_key = HiddenServiceSecretKey::from(&expanded_secret_key);

    let directory = generate
        .output
        .as_ref()
        .map_or_else(Default::default, Cow::Borrowed);

    File::create(directory.join("hostname"))
        .unwrap()
        .write_all(&Vec::<u8>::from(&hostname))
        .unwrap();

    File::create(directory.join("hs_ed25519_public_key"))
        .unwrap()
        .write_all(&Vec::<u8>::from(&hidden_service_public_key))
        .unwrap();

    File::create(directory.join("hs_ed25519_secret_key"))
        .unwrap()
        .write_all(&Vec::<u8>::from(&hidden_service_secret_key))
        .unwrap();
}
