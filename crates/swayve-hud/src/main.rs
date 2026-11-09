use std::env;
use std::path::PathBuf;
use std::process;

use swayve_hud::client::{resolve_socket_path, run_client};

fn print_help() {
    println!("swayve-hud: live terminal diff consumer for Swayve");
    println!();
    println!("USAGE:");
    println!("    swayve-hud [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help           Print help information");
    println!("    --socket <PATH>      Override daemon socket path");
    println!();
    println!("ENVIRONMENT:");
    println!("    SWAYVE_SOCKET        Unix domain socket path to Swayve daemon");
    println!("    XDG_RUNTIME_DIR      Fallback directory for swayve/swayve.sock");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut custom_socket = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "--socket" => {
                if i + 1 < args.len() {
                    custom_socket = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                } else {
                    eprintln!("Error: --socket requires a path argument");
                    process::exit(1);
                }
            }
            other => {
                eprintln!("Error: unknown argument '{other}'");
                eprintln!("Run 'swayve-hud --help' for usage.");
                process::exit(1);
            }
        }
        i += 1;
    }

    let socket_path = match custom_socket {
        Some(p) => p,
        None => match resolve_socket_path() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        },
    };

    if let Err(e) = run_client(&socket_path) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
