use putpng::grab::*;
use putpng::crop::*;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    commands: Commands,

    //Paths to be used
    #[arg(global=true)]
    paths: Vec<String>,

    ///If paths contain these strings, ignore them
    #[arg(short, long, global=true, num_args = 1 ..)]
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

fn ignoring(ignore: Vec<String>) -> impl Fn(&String) -> bool {
    move |p| !ignore.iter().any(|i| p.contains(i))
}

fn main() {
    let args = Args::parse_from(wild::args());

    let (commands, paths) = (args.commands, args.paths.into_iter().filter(ignoring(args.ignore)));
    match commands {
        Commands::Grab {x, y} => {
            let _ = grab_all(paths, x, y).inspect_err(|e| eprintln!("{e}"));
        },
        Commands::Crop => {
            let _ = apply_crop(paths).inspect_err(|e| eprintln!("{e}"));
        },
        Commands::Show => for path in paths {
            match read_grab_offset(&path) {
                Ok(Some(offset)) => println!("'{path}': {offset:?}"),
                Err(e) => eprintln!("{e}"),
                _ => println!("'{path}' does not have an offset")
            }
        }
    }
}