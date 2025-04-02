use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use validations::has_sudo_access;

mod setup;
mod validations;

/// Program by deployerra to manage deployements on the server side.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    //// Password for sudo access (only required if not already available).
    #[arg(short, long)]
    password: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Setup the server for deploying applications.
    Setup,
}

fn main() {
    let args = Args::parse();

    let distro = match validations::check_distro() {
        Ok(distro) => {
            println!("supported distro detected: {}", distro);
            distro
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    if !has_sudo_access() {
        match &args.password {
            Some(_password) => {
                println!("using provided password for sudo access");
                process::exit(1); // Temporarily exit to avoid further execution, further implementation needed
            }
            None => {
                eprintln!(
                    "Sudo access is required. Please provide a password using the -p or --password flag."
                );
                process::exit(1);
            }
        }
    } else {
        println!("sudo access confirmed. continuing execution");
    }

    match args.command {
        Some(Commands::Setup) => {
            setup::setup(distro);
        }
        None => {
            println!("No command was provided! Please read the following instructions:\n");
            let mut cmd = Args::command();
            let _ = cmd.print_help();
        }
    }
}
