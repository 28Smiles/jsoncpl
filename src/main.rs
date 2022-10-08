use std::fs;
use clap::Parser;
use crate::checks::entry_parity::entry_parity;
use crate::checks::file_parity::file_parity;
use crate::checks::file_style::file_style;
use crate::cli::{Cli, Commands, Indentation};
use crate::io::read_folders;
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

fn lint(cli: Cli) {
    let folders = read_folders(&cli.folders);
    let (file_types, errors) = file_parity(folders);
    println!("{}", errors.join("\n\n"));

    let style = cli_to_style(&cli);
    for file_type in file_types {
        let loaded_files = file_type.into_iter()
            .map(|file| file.load())
            .collect::<Vec<_>>();
        let mut jsons = loaded_files.iter().map(|file| {
            let (json, errors) = file_style(&style, file).unwrap();
            println!("{}", errors.join("\n\n"));

            (file, json)
        }).collect::<Vec<_>>();
        println!("{}", entry_parity(&mut jsons).join("\n\n"));
    }
}

fn format(cli: Cli) {
    let folders = read_folders(&cli.folders);
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

    match cli.command {
        Commands::Format => {
            format(cli);
        }
        Commands::Lint => {
            lint(cli);
        }
    }
}
