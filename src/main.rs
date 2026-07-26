use anyhow::Result;
use clap::Parser;
use cocox::cli::Cli;
use cocox::command;

fn main() -> Result<()> {
    let args = Cli::parse();
    command::run(args)
}
