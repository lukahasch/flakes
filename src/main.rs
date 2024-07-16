use clap::Parser;
use flakes::flake;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: Option<String>,

    #[clap(short, long)]
    arg: Option<String>,

    #[clap(short, long)]
    others: Option<Vec<String>>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let code = std::fs::File::open(args.file.unwrap_or("flake.lua".to_string()))?;
    let input = serde_json::from_str(&args.arg.unwrap_or("{}".to_string()))?;
    if args.others.is_some() {
        todo!("others");
    }
    flake(code, &input).map(|output| {
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    })
}
