use std::{fs::File, io::Write};

use tor_operator::{
    cli::{
        parse, CliArgs, CliCommands, ControllerArgs, ControllerCommands, ControllerRunArgs,
        CrdArgs, CrdCommands, CrdGenerateArgs, CrdGenerateArgsFormat,
    },
    controller, crd, http_server,
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

    let busybox_image_pull_policy = &run.busybox_image_pull_policy;
    let busybox_image_uri = &run.busybox_image_uri;
    let tor_image_pull_policy = &run.tor_image_pull_policy;
    let tor_image_uri = &run.tor_image_uri;

    let http_server = http_server::run(addr);
    let controller = controller::run(
        busybox_image_pull_policy,
        busybox_image_uri,
        tor_image_pull_policy,
        tor_image_uri,
    );

    tokio::select! {
        _ = http_server => {},
        _ = controller => {},
    }
}

fn crd_generate(_cli: &CliArgs, _crd: &CrdArgs, generate: &CrdGenerateArgs) {
    let crd = crd::generate_onion_service();

    let content = match generate.format {
        CrdGenerateArgsFormat::Json => serde_json::to_string_pretty(&crd).unwrap(),
        CrdGenerateArgsFormat::Yaml => serde_yaml::to_string(&crd).unwrap(),
    };

    if let Some(output) = &generate.output {
        let path = match generate.format {
            CrdGenerateArgsFormat::Json => output.join("onionservice.json"),
            CrdGenerateArgsFormat::Yaml => output.join("onionservice.yaml"),
        };

        File::create(path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
    } else {
        print!("{content}");
    }
}
