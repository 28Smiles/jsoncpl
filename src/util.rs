use std::ops::Range;
use colored::Colorize;

pub fn print_lines(display_lines: Range<u32>, content: &str) -> String {
    let mut buffer = String::new();
    for (i, line_content) in content.lines().enumerate() {
        let i = i + 1;
        if display_lines.start - 2 <= i as u32 && i as u32 <= display_lines.end + 2 {
            buffer = format!(
                "{} [{}]{}\n",
                buffer,
                format!("{}", i).blue(),
                if display_lines.start - 1 < (i as u32) && (i as u32) < display_lines.end + 1 {
                    line_content.red()
                } else {
                    line_content.normal()
                }
            );
        }
    }

    buffer.pop();
    buffer
}
