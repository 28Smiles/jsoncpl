use colored::Colorize;
use crate::io::LoadedFile;
use crate::parser::model::{JsonObject, JsonStyle};
use crate::parser::parser::Parser;
use crate::util::print_lines;

pub fn file_style<'a, 'b, 'c>(
    style: &'b JsonStyle,
    file: &'a LoadedFile,
    errors: &mut Vec<String>,
) -> Result<JsonObject<'a>, ()> {
    match Parser::new(*style).parse(file.content()) {
        Ok((json, style_errors)) => {
            for (span, error_text) in style_errors {
                let line = span.location_line();
                errors.push(format!(
                    "[{}] {}\n {}\n{}",
                    format!("STYLE").yellow(),
                    error_text,
                    format!("{}", file.path().to_str().unwrap()).green(),
                    print_lines(line..line, file.content()),
                ));
            }
            Ok(json)
        },
        Err(e) => {
            errors.push(format!(
                "[{}] {}\n{}",
                format!("ERROR").red(),
                "Can not parse json",
                e
            ));
            Err(())
        }
    }
}
