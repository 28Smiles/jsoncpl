use std::path::PathBuf;
use clap::{ValueEnum, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = "A tool for linting and formatting json files")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// The expected sorting algorithm for keys in the json file
    #[arg(short, long, value_enum, default_value = "default")]
    pub algorithm: SortAlgorithm,

    /// The expected sort order for keys in the json file
    #[arg(short, long, value_enum, default_value = "asc")]
    pub order: SortOrder,

    /// The expected line endings of the json file
    #[arg(short, long, value_enum, default_value = "lf")]
    pub line_endings: LineEnding,

    /// The expected indentation of the json files
    #[arg(short, long, default_value = "four")]
    pub indent: Indentation,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Subcommand)]
pub enum Commands {
    /// Format the provided files according to the style parameters
    Format {
        /// List the folders to search for files to format
        folders: Vec<PathBuf>,
    },
    /// Check the provided files according to the style parameters
    Lint {
        /// List the folders to search for files to lint and compare
        folders: Vec<PathBuf>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SortAlgorithm {
    /// Sort the keys by natural sort
    Natural,
    /// Sort the keys by classical sort
    Default,
    /// Ignore sort order
    IGNORE
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SortOrder {
    /// Sort the keys by ascending order
    Asc,
    /// Sort the keys by descending order
    Desc
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LineEnding {
    /// Add \r\n to the end of an entry
    CRLF,
    /// Add \n to the end of an entry
    LF,
    /// Add no linebreaks
    NONE,
    /// Accept any linebreak (\r\n or \n) (defaults to \n in formatting)
    ANY,
    /// Accept anything (defaults to \n in formatting)
    IGNORE
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Indentation {
    /// Indent with \t
    TAB,
    /// Indent with "  "
    TWO,
    /// Indent with "    "
    FOUR,
    /// Ignore indentation
    IGNORE,
}
