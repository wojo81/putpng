use clap::{Parser, Subcommand};
use putpng::crc::*;
use putpng::crop::*;
use putpng::grab::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    override_usage = "putpng <COMMAND> [PATHS]... [IGNORE-OPTION]"
)]
struct Args {
    #[command(subcommand)]
    commands: Commands,

    //Paths to be used
    #[arg(global = true)]
    paths: Vec<PathBuf>,

    ///If paths contain these strings, ignore them
    #[arg(short, long, global = true, num_args = 1..)]
    ignore: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    ///Apply the given offsets to the specified png files
    Grab { x: String, y: String },
    ///Crop out the empty edges of the specified png files, but keep the relative offset
    Crop,
    ///Displays the grab offsets of the specified png files
    Show,
}

fn ignoring(ignore: Vec<String>) -> impl Fn(&&PathBuf) -> bool {
    move |p| !ignore.iter().any(|i| p.to_string_lossy().contains(i))
}

fn main() {
    let args = Args::parse_from(wild::args());

    let (commands, paths) = (
        args.commands,
        args.paths
            .iter()
            .filter(ignoring(args.ignore))
            .map(PathBuf::as_path),
    );
    match commands {
        Commands::Grab { x, y } => {
            let crc = Crc32::new();
            let _ = grab_all(paths, &crc, &x, &y, false).inspect_err(|e| eprintln!("{e}"));
        }
        Commands::Crop => {
            let crc = Crc32::new();
            let _ = crop_all(paths, &crc).inspect_err(|e| eprintln!("{e}"));
        }
        Commands::Show => {
            for path in paths {
                match read_grab(path) {
                    Ok(Some(offset)) => println!("{path:?}: {offset:?}"),
                    Err(e) => eprintln!("{e}"),
                    _ => println!("{path:?} does not have an offset"),
                }
            }
        }
    }
}
