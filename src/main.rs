use std::{fs::File, io::Write};

use tor_operator::{
    cli::{parse, CliArgs, Commands, CrdArgs, CrdCommands, CrdGenerateArgs, CrdGenerateArgsFormat},
    crd,
};

fn main() {
    let cli_args = &parse();

    match &cli_args.command {
        Commands::Crd(crd_args) => match &crd_args.command {
            CrdCommands::Generate(crd_generate_args) => {
                crd_generate(cli_args, crd_args, crd_generate_args);
            }
        },
    }
}

fn crd_generate(_cli_args: &CliArgs, _crd_args: &CrdArgs, crd_generate_args: &CrdGenerateArgs) {
    let crd = crd::generate();

    let content = match crd_generate_args.format {
        CrdGenerateArgsFormat::Json => serde_json::to_string_pretty(&crd).unwrap(),
        CrdGenerateArgsFormat::Yaml => serde_yaml::to_string(&crd).unwrap(),
    };

    if let Some(output) = &crd_generate_args.output {
        File::create(output)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
    } else {
        print!("{content}");
    }
}
