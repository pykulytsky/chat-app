use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub user: String,
    #[arg(short, long)]
    pub color: Option<String>,
    #[arg(short, long)]
    pub room: Option<String>,
}
