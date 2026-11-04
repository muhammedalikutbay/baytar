use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "baytar-cli", about = "Baytar development tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the Baytar core version.
    Version,
}

fn main() {
    match Cli::parse().command {
        Commands::Version => println!("{}", baytar_core::version()),
    }
}
