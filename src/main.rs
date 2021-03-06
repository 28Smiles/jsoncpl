#[macro_use]
extern crate colour;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use clap::{ArgEnum, Parser, IntoApp, ErrorKind};

use crate::parser::{JsonObject, JsonString, JsonType, Pretty};
use std::process::exit;

mod parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = "A tool for linting json files")]
struct Cli {
    #[clap(parse(from_os_str), help = "List the folders to search for files to lint and compare")]
    folders: Vec<PathBuf>,

    #[clap(short, long, arg_enum, default_value = "default", help = "The expected sorting algorithm for keys in the json file")]
    sort: SortingMode,

    #[clap(short, long, arg_enum, default_value = "asc", help = "The expected sort order for keys in the json file")]
    order: Order,

    #[clap(long, help = "Whether the files should be automatically formatted")]
    format: bool,

    #[clap(long, help = "Whether the style check should be skipped")]
    skip_check_style: bool,

    #[clap(long, help = "Whether the key order check should be skipped")]
    skip_check_order: bool,

    #[clap(long, help = "Whether the key parity check should be skipped")]
    skip_check_parity: bool,

    #[clap(long, default_value = "4", help = "The expected indentation of the json files")]
    indent: i32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum SortingMode {
    Natural,
    Default
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Order {
    Asc,
    Desc
}

fn file_tree(base_path: &PathBuf, path: &PathBuf, mut files: Vec<PathBuf>) -> Vec<PathBuf> {
    for entry in path.read_dir().expect(&*format!("\"{}\" is not a directory", path.as_os_str().to_str().unwrap())) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            files = file_tree(base_path, &path, files);
        } else {
            files.push(pathdiff::diff_paths(path, base_path).unwrap());
        }
    }

    files
}


fn read_folders(args: &Cli) -> Vec<(&PathBuf, Vec<PathBuf>)> {
    let mut lang_content: Vec<(&PathBuf, Vec<PathBuf>)> = Vec::new();
    for path in &args.folders {
        let content = file_tree(
            path,
            path,
            Vec::new()
        );
        lang_content.push((path, content));
    }

    lang_content
}

fn check_file_parity(files: &Vec<(&PathBuf, Vec<PathBuf>)>) -> bool {
    let mut reference_content: BTreeSet<&PathBuf> = BTreeSet::new();
    let mut success = true;
    println!("Checking file parity in folders...");
    println!("Will look for following files:");
    for (_, files) in files {
        for file_path in files {
            if reference_content.insert(file_path) {
                yellow_ln!(" - '{}'", file_path.to_str().unwrap())
            }
        }
    }

    for (folder_path, files) in files {
        println!("Folder '{}':", folder_path.to_str().unwrap());
        let mut rcc = reference_content.clone();
        if files.len() > 0 {
            println!("  Found files:");
            for path in files {
                green_ln!("   - '{}'", path.to_str().unwrap());
                rcc.remove(path);
            }
        }
        if rcc.len() > 0 {
            success = false;
            println!("  Missing files:");
            for path in rcc {
                red_ln!("   - '{}'", &path.to_str().unwrap());
            }
        }
    }

    success
}

fn format<'a>(json: &JsonType<'a>, sorting_mode: &SortingMode, order: &Order) -> JsonType<'a> {
    match json {
        JsonType::Object(dict) => {
            let mut values: Vec<(JsonString, JsonType)> = dict.values.iter()
                .map(|(key, value)| {
                    (*key, format(value, sorting_mode, order))
                }).collect();

            match sorting_mode {
                SortingMode::Natural =>
                    values.sort_by(|(ka, _), (kb, _)| human_sort::compare(ka.value, kb.value)),
                SortingMode::Default =>
                    values.sort_by(|(ka, _), (kb, _)| ka.value.cmp(kb.value)),
            }
            if let Order::Desc = order  {
                values.reverse();
            }

            JsonType::Object(JsonObject {
                values: values,
                position: dict.position
            })
        }
        JsonType::String(string) => {
            JsonType::String(*string)
        }
    }
}

fn lint(json: &JsonType, sorting_mode: &SortingMode, order: &Order) -> bool {
    if let JsonType::Object(dict) = json {
        let ordered_dict = &dict.values;
        let mut ordered_dict: Vec<&JsonString> = ordered_dict.into_iter().map(|(key, _)| key).collect();
        match sorting_mode {
            SortingMode::Natural =>
                ordered_dict.sort_by(|a, b| human_sort::compare(a.value, b.value)),
            SortingMode::Default =>
                ordered_dict.sort_by(|a, b| a.cmp(b)),
        }
        if let Order::Desc = order  {
            ordered_dict.reverse();
        }

        let original_dict = &dict.values;
        let mut success = true;
        for ((original, value), expected) in original_dict.into_iter().zip(ordered_dict) {
            if original != expected {
                success = false;
                red_ln!(
                    "    Error in line {}, column {} expected \"{}\", found \"{}\" which is in line {}, column {}",
                    original.position.location_line(),
                    original.position.get_utf8_column(),
                    expected.value,
                    original.value,
                    expected.position.location_line(),
                    expected.position.get_utf8_column()
                )
            }
            let lint_result = lint(value, sorting_mode, order);
            success = success && lint_result;
        }

        success
    } else {
        true
    }
}

fn format_files(args: &Cli, folders: &Vec<(&PathBuf, Vec<(&PathBuf, JsonType)>)>) -> bool {
    println!("Format files in folders...");
    let mut success = true;
    for (folder_path, file_paths) in folders {
        println!("Checking folder '{}':", folder_path.to_str().unwrap());
        for (file_path, json) in file_paths {
            println!("  Checking file '{}':", file_path.to_str().unwrap());
            let mut path = (*folder_path).clone();
            path.push(file_path);

            let pretty = format(json, &args.sort, &args.order).pretty(args.indent, args.indent);
            let write_success = fs::write(&path, &*pretty).is_ok();
            success = success && write_success;
        }
    }

    success
}

fn check_key_order(folders: &Vec<(&PathBuf, Vec<(&PathBuf, JsonType)>)>, sorting_mode: &SortingMode, order: &Order) -> bool {
    let mut success = true;
    println!("Checking key order in folders...");
    for (folder_path, file_paths) in folders {
        println!("Checking folder '{}':", folder_path.to_str().unwrap());
        for (file_path, json) in file_paths {
            println!("  Checking file '{}':", file_path.to_str().unwrap());

            let lint_success = lint(&json, sorting_mode, order);
            success = success && lint_success;
            if lint_success {
                green_ln!("    Ok!");
            }
        }
    }

    success
}

fn read_files<'a, 'b>(file_paths: &'b Vec<(&'a PathBuf, Vec<PathBuf>)>) -> Vec<(&'a PathBuf, Vec<(&'b PathBuf, String)>)> {
    file_paths.iter().map(|(folder_path, file_paths)| {
        (*folder_path, file_paths.iter().map(|file_path| {
            let mut path = (*folder_path).clone();
            path.push(file_path);

            if let Ok(file) = fs::read_to_string(&path) {
                Some((file_path, file))
            } else {
                red_ln!("Error in reading file \"{}\"", path.to_str().unwrap());
                None
            }
        }).filter(|option| option.is_some()).map(|option| option.unwrap()).collect())
    }).collect()
}

fn check_file_style(original_files: &Vec<(&PathBuf, Vec<(&PathBuf, String)>)>, parsed_files: &Vec<(&PathBuf, Vec<(&PathBuf, JsonType)>)>, indent: i32) -> bool {
    let mut success = true;
    println!("Checking file style in folders...");
    for ((original_folder_path, file_contents), (parsed_folder_path, parsed_files)) in original_files.iter().zip(parsed_files) {
        assert_eq!(original_folder_path.to_str().unwrap(), parsed_folder_path.to_str().unwrap());
        println!("Checking folder '{}':", original_folder_path.to_str().unwrap());
        'next_file: for ((original_file_path, file_content), (parsed_file_path, parsed_file)) in file_contents.iter().zip(parsed_files.iter()) {
            assert_eq!(original_file_path.to_str().unwrap(), parsed_file_path.to_str().unwrap());
            println!("  Checking file '{}':", original_file_path.to_str().unwrap());
            let pretty = parsed_file.pretty(indent, indent);
            let mut line = 0;
            let mut col = 0;
            for (reference, found) in pretty.chars().zip(file_content.chars()) {
                if reference != found {
                    red_ln!("    Error in line {} and col {}, expected {}, found {}", line + 1, col, reference, found);
                    success = false;
                    continue 'next_file;
                }
                col = col + 1;
                if reference == '\n' {
                    line = line + 1;
                    col = 0;
                }
            }
            green_ln!("    OK!");
        }
    }

    success
}

fn parse_files<'s>(file_paths: &'s Vec<(&'s PathBuf, Vec<(&'s PathBuf, String)>)>) -> Vec<(&'s PathBuf, Vec<(&'s PathBuf, JsonType<'s>)>)> {
    file_paths.iter().map(|(folder_path, file_paths)| {
        (*folder_path, file_paths.iter().map(|(file_path, file_content)| {
            if let Some(file_json) = parser::parse_root(file_content) {
                Some((*file_path, JsonType::Object(file_json)))
            } else {
                let mut path = folder_path.to_owned().clone();
                path.push(file_path);
                red_ln!("Error parsing file \"{}\"", path.to_str().unwrap());
                None
            }
        }).filter(|option| option.is_some()).map(|option| option.unwrap()).collect())
    }).collect()
}

fn check_json<'a>(parent_key: &str, reference: &'a JsonType<'a>, comparison: &JsonType, parents: &mut Vec<&'a JsonString<'a>>) -> bool {
    match reference {
        JsonType::Object(reference_values) => {
            match comparison {
                JsonType::Object(values) => {
                    let mut success = true;
                    'references: for (reference_key, reference_value) in &reference_values.values {
                        for (key, value) in &values.values {
                            if key == reference_key {
                                parents.push(reference_key);
                                let inner_success = check_json(key.value, reference_value, value, parents);
                                parents.pop().unwrap();
                                success = success && inner_success;

                                continue 'references;
                            }
                        }
                        // Did not break -> Not found
                        parents.push(reference_key);
                        red_ln!("    Key {} not found", parents.iter().map(|k| k.value).collect::<Vec<&str>>().join("."));
                        parents.pop().unwrap();
                        success = false;
                    }

                    success
                },
                JsonType::String(string) => {
                    red_ln!("    Type mismatch, expected type object for key \"{}\" in line {} and column {} but found a string", parent_key, string.position.location_line(), string.position.get_column());
                    false
                }
            }
        }
        JsonType::String(_) => {
            match comparison {
                JsonType::Object(object) => {
                    red_ln!("    Type mismatch, expected type string for key \"{}\" in line {} and column {} but found an object", parent_key, object.position.location_line(), object.position.get_column());
                    false
                },
                JsonType::String(_) => true
            }
        }
    }
}

fn combine_json<'a>(a: &JsonType<'a>, b: &JsonType<'a>, file: &PathBuf) -> JsonType<'a> {
    match a {
        JsonType::Object(a_object) => {
            match b {
                JsonType::Object(b_object) => {
                    let mut combined_keys = BTreeSet::new();
                    for (a_key, _) in &a_object.values {
                        combined_keys.insert(a_key);
                    }
                    for (b_key, _) in &b_object.values {
                        combined_keys.insert(b_key);
                    }

                    let mut combined = Vec::new();
                    'keys: for key in combined_keys {
                        for (a_key, a_value) in &a_object.values {
                            if key == a_key {
                                for (b_key, b_value) in &b_object.values {
                                    if key == b_key {
                                        combined.push((key.clone(), combine_json(a_value, b_value, file)));
                                        continue 'keys;
                                    }
                                }

                                // A
                                combined.push((key.clone(), a_value.clone()));
                                continue 'keys;
                            }
                        }
                        for (b_key, b_value) in &b_object.values {
                            if key == b_key {
                                // B
                                combined.push((key.clone(), b_value.clone()));
                                continue 'keys;
                            }
                        }
                    }

                    JsonType::Object(JsonObject {
                        values: combined,
                        position: a_object.position
                    })
                }
                JsonType::String(b_string) => {
                    red_ln!(
                        "    Type mismatch, expected type object, but found type string in file {} in line {} column {}",
                        file.to_str().unwrap(),
                        b_string.position.location_line(),
                        b_string.position.get_column()
                    );
                    JsonType::Object(a_object.clone())
                }
            }
        }
        JsonType::String(a_string) => {
            match b {
                JsonType::Object(b_object) => {
                    red_ln!(
                        "    Type mismatch, expected type object, but found type string in file {} in line {} column {}",
                        file.to_str().unwrap(),
                        b_object.position.location_line(),
                        b_object.position.get_column()
                    );
                    JsonType::Object(b_object.clone())
                }
                JsonType::String(_) => {
                    JsonType::String(*a_string)
                }
            }
        }
    }
}

fn check_key_parity(folders: &Vec<(&PathBuf, Vec<(&PathBuf, JsonType)>)>) -> bool {
    println!("Checking key parity folders...");
    let mut reference_content: BTreeMap<&PathBuf, JsonType> = BTreeMap::new();
    let mut success = true;
    for (_, file_paths) in folders {
        for (file_path, json) in file_paths {
            if !reference_content.contains_key(*file_path) {
                reference_content.insert(*file_path, json.clone());
            } else {
                let reference_json = reference_content.remove(*file_path).unwrap();
                let reference_json = combine_json(&reference_json, json, *file_path);
                reference_content.insert(*file_path, reference_json);
            }
        }
    }

    for (folder_path, file_paths) in folders {
        println!("Checking folder '{}':", folder_path.to_str().unwrap());
        let folder_reference_content = reference_content.clone();
        for (file_path, json) in file_paths {
            println!("  Checking file '{}':", file_path.to_str().unwrap());
            let reference = folder_reference_content.get(*file_path).unwrap();
            let mut stack = Vec::new();
            let inner_success = check_json("<root>", reference, json, &mut stack);

            success = success && inner_success;
            if inner_success {
                green_ln!("    Ok!");
            }
        }
    }
    success
}

fn main() {
    let args: Cli = Cli::parse();
    if args.folders.len() == 0 {
        let mut app = Cli::command();
        app.error(
            ErrorKind::ArgumentConflict,
            "You need to specify at least one folder",
        ).exit();
    }

    let files = read_folders(&args);
    let file_parity_success = args.skip_check_parity || check_file_parity(&files);
    let contents = read_files(&files);
    let jsons = parse_files(&contents);
    if args.format {
        format_files(&args, &jsons);
    }
    let file_style_success = args.format || args.skip_check_style || check_file_style(&contents, &jsons, args.indent);
    let key_order_success = args.format || args.skip_check_order || check_key_order(&jsons, &args.sort, &args.order);
    let key_parity_success = args.skip_check_parity || check_key_parity(&jsons);

    let success = file_parity_success && file_style_success && key_order_success && key_parity_success;

    if success {
        green_ln!("\nLint Successful");
    } else {
        red_ln!("\nLint Failed");
        exit(-1);
    }
}
