use std::ops::Deref;
use colored::Colorize;
use crate::io::LoadedFile;
use crate::parser::model::{JsonObject, JsonString, JsonType};
use crate::util::print_lines;

fn join<'a, 'b: 'a>(
    warnings: &mut Vec<(JsonString<'a>, JsonType<'a>, JsonType<'b>)>,
    left: &mut JsonObject<'a>,
    right: &JsonObject<'b>
) {
    for (r_key, r_value) in &right.values {
        if let Some((_, l_value)) = left.values.iter_mut().find(
            |(l_key, _)| {
                r_key.value == l_key.value
        }) {
            match (l_value, r_value) {
                (JsonType::Object(l_object), JsonType::Object(r_object)) => {
                    join(warnings, l_object, r_object);
                }
                (JsonType::String(_), JsonType::String(_)) => {}
                (l, r) => {
                    warnings.push((r_key.clone(), l.clone(), r.clone()));
                }
            }
        } else {
            left.values.push((*r_key, r_value.clone()))
        }
    }
}

fn compare<'a, 'b>(
    warnings: &mut Vec<Vec<JsonString<'a>>>,
    acc: &JsonObject<'a>,
    object: &JsonObject<'b>,
    path: &Vec<JsonString<'a>>,
) {
    for (acc_key, acc_value) in &acc.values {
        if let Some((_, value)) = object.values.iter().find(|(key, _)| acc_key.value == key.value) {
            match (acc_value, value) {
                (JsonType::Object(acc_obj), JsonType::Object(obj)) => {
                    let mut path = path.clone();
                    path.push(acc_key.clone());
                    compare(warnings, acc_obj, obj, &path);
                },
                _ => {},
            }
        } else {
            let mut path = path.clone();
            path.push(acc_key.clone());
            warnings.push(path);
        }
    }
}

pub fn entry_parity<'a>(files: &Vec<(&'a LoadedFile, JsonObject<'a>)>) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(((_, l_object), right)) = files.split_first() {
        let mut acc = l_object.clone();
        for (r_file, r_object) in right {
            let mut join_warnings = Vec::new();
            join(&mut join_warnings, &mut acc, r_object);

            for (key, l_type, r_type) in join_warnings {
                match (l_type, r_type) {
                    (JsonType::Object(l_object), JsonType::String(r_string)) => {
                        let (l_file, _) = files.iter()
                            .find(|(_, object)| *object.start.deref() == *l_object.start.deref())
                            .unwrap();
                        let l_path = l_file.path();
                        let l_content = l_file.content();
                        let r_path = r_file.path();
                        let r_content = r_file.content();

                        warnings.push(format!(
                            "[{}] Found different value types for key \"{}\"\n{}\n{}\n{}\n{}",
                            "PAIRITY".yellow(),
                            key.value,
                            l_path.to_str().unwrap().green(),
                            print_lines(l_object.start.location_line()..l_object.end.location_line(), l_content),
                            r_path.to_str().unwrap().green(),
                            print_lines(r_string.start.location_line()..r_string.end.location_line(), r_content),
                        ));
                    }
                    (JsonType::String(l_string), JsonType::Object(r_object)) => {
                        let (l_file, _) = files.iter()
                            .find(|(_, object)|
                                *object.start.deref() == *l_object.start.deref()
                            )
                            .unwrap();
                        let l_path = l_file.path();
                        let l_content = l_file.content();
                        let r_path = r_file.path();
                        let r_content = r_file.content();

                        warnings.push(format!(
                            "[{}] Found different value types for key \"{}\"\n{}\n{}\n{}\n{}",
                            "PAIRITY".yellow(),
                            key.value,
                            l_path.to_str().unwrap().green(),
                            print_lines(l_string.start.location_line()..l_string.end.location_line(), l_content),
                            r_path.to_str().unwrap().green(),
                            print_lines(r_object.start.location_line()..r_object.end.location_line(), r_content),
                        ));
                    }
                    _ => panic!(),
                }
            }
        }

        for (file, object) in files {
            let mut compare_warnings = Vec::new();
            compare(&mut compare_warnings, &acc, object, &Vec::new());
            for compare_warning in compare_warnings {
                warnings.push(format!(
                    "[{}] Cant find key `{}` in file {}",
                    "PAIRITY".yellow(),
                    compare_warning.iter()
                        .map(|p| format!("\"{}\"", p.value))
                        .collect::<Vec<_>>()
                        .join(".").blue(),
                    file.path().to_str().unwrap().green(),
                ));
            }
        }
    }

    warnings
}
