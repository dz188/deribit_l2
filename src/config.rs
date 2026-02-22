use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Config {
    /// Instruments to subscribe to (comma separated)
    #[arg(short, long)]
    pub instruments: Vec<String>,

    /// Output mode: console or file
    #[arg(short, long, default_value = "console")]
    pub output: String,

    /// File path if output is file
    #[arg(long)]
    pub file: Option<String>,
}
