use clap::{ArgAction, ArgGroup, Parser};

#[derive(Debug, Parser)]
#[command(name = "cocox")]
#[command(about = "Check if a commit message follows the conventional commit format.")]
#[command(version)]
#[command(group(
    ArgGroup::new("input")
        .args(["message", "file", "hash", "from_hash"])
        .required(true)
        .multiple(false)))]
pub struct Cli {
    #[arg(help = "The commit message to be checked")]
    pub message: Option<String>,

    #[arg(long, help = "Path to a file containing the commit message")]
    pub file: Option<String>,

    #[arg(long, help = "Commit hash")]
    pub hash: Option<String>,

    #[arg(long = "from-hash", help = "From commit hash")]
    pub from_hash: Option<String>,

    #[arg(
        long = "to-hash",
        requires = "from_hash",
        conflicts_with_all = ["message", "file", "hash"],
        default_value = "HEAD",
        help = "To commit hash"
    )]
    pub to_hash: String,

    #[arg(
        long = "skip-detail",
        help = "Skip the detailed error message check",
        action = ArgAction::SetTrue
    )]
    pub skip_detail: bool,

    #[arg(
        long = "hide-input",
        help = "Hide input from stdout",
        action = ArgAction::SetTrue
    )]
    pub hide_input: bool,

    #[arg(
        short,
        long,
        help = "Ignore stdout and stderr",
        conflicts_with = "verbose",
        action = ArgAction::SetTrue
    )]
    pub quiet: bool,

    #[arg(
        short,
        long,
        help = "Verbose output",
        conflicts_with = "quiet",
        action = ArgAction::SetTrue
    )]
    pub verbose: bool,
}
