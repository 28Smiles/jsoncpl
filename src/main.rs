use std::fs;
use std::path::PathBuf;
use clap::Parser;
use crate::checks::entry_parity::entry_parity;
use crate::checks::file_parity::file_parity;
use crate::checks::file_style::file_style;
use crate::cli::{Cli, Commands, Indentation};
use crate::io::{File, read_folders};
use crate::parser::model::{JsonStyle, LineEnding, SortAlgorithm, SortOrder};

mod cli;
mod io;
mod parser;
mod checks;
mod util;
mod natural_sort;

fn cli_to_style(cli: &Cli) -> JsonStyle {
    JsonStyle::STYLED {
        post_colon: Some(" "),
        indentation: match cli.indent {
            Indentation::TAB => Some("\t"),
            Indentation::TWO => Some("  "),
            Indentation::FOUR => Some("    "),
            Indentation::IGNORE => None,
        },
        line_endings: match cli.line_endings {
            cli::LineEnding::CRLF => LineEnding::CRLF,
            cli::LineEnding::LF => LineEnding::LF,
            cli::LineEnding::NONE => LineEnding::NONE,
            cli::LineEnding::ANY => LineEnding::ANY,
            cli::LineEnding::IGNORE => LineEnding::IGNORE,
        },
        order: match cli.order {
            cli::SortOrder::Asc => SortOrder::ASC,
            cli::SortOrder::Desc => SortOrder::DESC,
        },
        sort_algorithm: match cli.algorithm {
            cli::SortAlgorithm::Natural => SortAlgorithm::NATURAL,
            cli::SortAlgorithm::Default => SortAlgorithm::NORMAL,
            cli::SortAlgorithm::IGNORE => SortAlgorithm::NONE,
        }
    }
}

fn lint(cli: Cli, folders: Vec<(PathBuf, Vec<File>)>) -> bool {
    let (file_types, errors) = file_parity(folders);
    println!("{}", errors.join("\n"));

    let style = cli_to_style(&cli);
    for file_type in file_types {
        let loaded_files = file_type.into_iter()
            .map(|file| file.load())
            .collect::<Vec<_>>();
        let mut jsons = loaded_files.iter().map(|file| {
            let (json, errors) = file_style(&style, file).unwrap();
            println!("{}", errors.join("\n"));

            (file, json)
        }).collect::<Vec<_>>();
        println!("{}", entry_parity(&mut jsons).join("\n"));
    }

    errors.is_empty()
}

fn format(cli: Cli, folders: Vec<(PathBuf, Vec<File>)>) {
    let style = cli_to_style(&cli);
    for (_, files) in folders {
        for file in files {
            let file = file.load();
            let (parsed, _) = parser::parser::Parser::new(
                JsonStyle::IGNORE
            ).parse(file.content()).unwrap();
            let generated = parser::generator::Generator::new(style).generate(parsed);
            fs::write(file.path(), &generated).unwrap();
        }
    }
}

fn main() {
    let cli: Cli = Cli::parse();

    match &cli.command {
        Commands::Format { folders } => {
            let folders = read_folders(folders);
            format(cli, folders);
        }
        Commands::Lint { folders } => {
            let folders = read_folders(folders);
            if !lint(cli, folders) {
                panic!("Linting failed");
            }
        }
    }
}
