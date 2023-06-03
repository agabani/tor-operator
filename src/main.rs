use std::{fs::File, io::Write};

use tor_operator::{
    cli::{parse, CliArgs, Commands, CrdArgs, CrdCommands, CrdGenerateArgs},
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
    let crd = crd::generate_crd();

    let yaml = serde_yaml::to_string(&crd).unwrap();

    if let Some(output) = &crd_generate_args.output {
        let mut file = File::create(output).unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        return;
    }

    println!("{yaml}");
}
