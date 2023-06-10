use std::{fs::File, io::Write};

use tor_operator::{
    cli::{
        parse, CliArgs, CliCommands, ControllerArgs, ControllerCommands, ControllerRunArgs,
        CrdArgs, CrdCommands, CrdGenerateArgs, CrdGenerateArgsFormat,
    },
    http_server, onionbalance, onionservice,
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
    }
}

async fn controller_run(_cli: &CliArgs, _controller: &ControllerArgs, run: &ControllerRunArgs) {
    let addr = format!("{}:{}", run.host, run.port).parse().unwrap();

    let onionbalance_config = onionbalance::Config {};

    let onion_service_config = onionservice::Config {
        tor_image: onionservice::ImageConfig {
            pull_policy: run.tor_image_pull_policy.clone(),
            uri: run.tor_image_uri.clone(),
        },
    };

    tokio::select! {
        _ = http_server::run(addr) => {},
        _ = onionbalance::run_controller(onionbalance_config) => {},
        _ = onionservice::run_controller(onion_service_config) => {},
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
            onionbalance::generate_custom_resource_definition(),
        ),
        (
            "onionservice",
            onionservice::generate_custom_resource_definition(),
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
