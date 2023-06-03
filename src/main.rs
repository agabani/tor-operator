use std::{fs::File, io::Write};

use tor_operator::{
    cli::{parse, Commands, CrdCommands},
    crd::crd,
};

fn main() {
    let cli = parse();

    match cli.command {
        Commands::Crd(crd_args) => match crd_args.command {
            CrdCommands::Generate(crd_generate_args) => {
                let crd = serde_yaml::to_string(&crd()).unwrap();

                if let Some(output) = crd_generate_args.output {
                    let mut file = File::create(output).unwrap();
                    file.write_all(crd.as_bytes()).unwrap();
                } else {
                    println!("{crd}");
                }
            }
        },
    }
}
