use colored::Colorize;
use crate::io::LoadedFile;
use crate::parser::model::{JsonObject, JsonStyle};
use crate::parser::parser::Parser;
use crate::util::print_lines;

pub fn file_style<'a, 'b, 'c>(
    style: &'b JsonStyle,
    file: &'a LoadedFile,
) -> Result<(JsonObject<'a>, Vec<String>), String> {
    match Parser::new(*style).parse(file.content()) {
        Ok((json, style_errors)) => {
            Ok((json, style_errors.into_iter().map(|(span, error_text)| {
                let line = span.location_line();
                format!(
                    "[{}] {}\n {}\n{}",
                    format!("STYLE").yellow(),
                    error_text,
                    format!("{}", file.path().to_str().unwrap()).green(),
                    print_lines(line..line, file.content()),
                )
            }).collect()))
        },
        Err(e) => Err(format!(
            "[{}] {}\n{}",
            format!("ERROR").red(),
            "Can not parse json",
            e
        ))
    }
}
