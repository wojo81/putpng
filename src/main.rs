#![feature(seek_seek_relative)]

use putpng::grab::*;
use putpng::crop::*;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None, propagate_version = true)]
struct Cli {
    #[arg(short, long)]
    ignore: Vec<String>,

    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Grab {x: i32, y: i32, paths: Vec<String>},
    Crop {paths: Vec<String>}
}

fn not_containing(ignore: Vec<String>) -> impl Fn(&String) -> bool {
    move |p| ignore.iter().find(|&i| p.contains(i)).is_none()
}

fn main() {
    let cli = Cli::parse_from(wild::args());

    match cli {
        Cli {ignore, commands: Commands::Grab {x, y, paths}} => {
            apply_grab(paths.into_iter().filter(not_containing(ignore)), x, y);
        },
        Cli {ignore, commands: Commands::Crop {paths}} => {
            apply_crop(paths.into_iter().filter(not_containing(ignore)));
        }
    }
}