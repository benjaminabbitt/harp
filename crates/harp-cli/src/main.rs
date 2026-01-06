use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "harp")]
#[command(about = "Generate random names", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a random name (default if no command given)
    Generate,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Generate) | None => {
            println!("{}", harp_core::generate_name());
        }
    }
}
