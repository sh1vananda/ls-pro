use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();
    println!("You want to list the contents of: {:?}", args.path);
}