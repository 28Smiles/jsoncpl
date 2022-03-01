#[macro_use]
extern crate colour;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use clap::{ArgEnum, Parser, IntoApp, ErrorKind};
use serde_json::{Map, Value};

use crate::parser::{JsonObject, JsonString, JsonType};
use std::process::exit;

mod parser;

#[derive(Parser)]
struct Cli {
    folders: Vec<String>,

    #[clap(short, long, arg_enum, default_value = "default")]
    sort: SortingMode,

    #[clap(short, long, arg_enum, default_value = "asc")]
    order: Order,

    #[clap(long)]
    format: bool,

    #[clap(long)]
    skip_check_style: bool,

    #[clap(long)]
    skip_check_order: bool,

    #[clap(long)]
    skip_check_parity: bool,
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


fn read_folders(args: &Cli) -> Vec<(PathBuf, Vec<PathBuf>)> {
    let mut lang_content: Vec<(PathBuf, Vec<PathBuf>)> = Vec::new();
    for folder in &args.folders {
        let path = PathBuf::from(folder);
        let content = file_tree(
            &path,
            &path,
            Vec::new()
        );
        lang_content.push((path, content));
    }

    lang_content
}

fn check_file_parity(files: &Vec<(PathBuf, Vec<PathBuf>)>) -> bool {
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

fn format(json: Value, sorting_mode: &SortingMode, order: &Order) -> Value {
    if let Value::Object(dict) = json {
        let mut dict: Vec<(String, Value)> = dict.into_iter().collect();
        match sorting_mode {
            SortingMode::Natural =>
                dict.sort_by(|(ka, _), (kb, _)| human_sort::compare(ka, kb)),
            SortingMode::Default =>
                dict.sort_by(|(ka, _), (kb, _)| ka.cmp(kb)),
        }
        if let Order::Desc = order  {
            dict.reverse();
        }
        let mut map = Map::with_capacity(dict.len());
        for (k, v) in dict {
            map.insert(k, format(v, sorting_mode, order));
        }

        Value::Object(map)
    } else {
        json
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
                ordered_dict.sort_by(|a, b| a.value.cmp(b.value)),
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

fn format_files(args: &Cli, folders: &Vec<(PathBuf, Vec<PathBuf>)>) -> bool {
    println!("Format files in folders...");
    let mut success = true;
    for (folder_path, file_paths) in folders {
        println!("Checking folder '{}':", folder_path.to_str().unwrap());
        for file_path in file_paths {
            println!("  Checking file '{}':", file_path.to_str().unwrap());
            let mut path = folder_path.clone();
            path.push(file_path);

            if let Ok(file_content) = fs::read_to_string(&path) {
                if let Ok(file_json) = serde_json::from_str(&*file_content) {
                    let value = format(file_json, &args.sort, &args.order);
                    if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                        let write_success = fs::write(&path, &*pretty).is_ok();
                        success = success && write_success;
                    } else {
                        red_ln!("Could not reserialize file: {}", &path.to_str().unwrap());
                        success = false;
                    }
                } else {
                    red_ln!("Could not parse file: {}", &path.to_str().unwrap());
                    success = false;
                }
            } else {
                red_ln!("Could not read file: {}", &path.to_str().unwrap());
                success = false;
            }
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
                green_ln!("    Success");
            }
        }
    }

    success
}

fn read_files(file_paths: &Vec<(PathBuf, Vec<PathBuf>)>) -> Vec<(&PathBuf, Vec<(&PathBuf, String)>)> {
    file_paths.iter().map(|(folder_path, file_paths)| {
        (folder_path, file_paths.iter().map(|file_path| {
            let mut path = folder_path.clone();
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

fn check_file_style(file_paths: &Vec<(&PathBuf, Vec<(&PathBuf, String)>)>) -> bool {
    let mut success = true;
    println!("Checking file style in folders...");
    for (folder_path, file_paths) in file_paths {
        println!("Checking folder '{}':", folder_path.to_str().unwrap());
        'next_file: for (file_path, file_content) in file_paths {
            println!("  Checking file '{}':", file_path.to_str().unwrap());
            if let Ok(file_json) = serde_json::from_str::<Value>(&*file_content) {
                if let Ok(pretty) = serde_json::to_string_pretty(&file_json) {
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
                } else {
                    red_ln!("Could not reserialize file");
                    success = false;
                }
            } else {
                red_ln!("Could not parse file");
                success = false;
            }
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
                            if key.value == reference_key.value {
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
                            if key.value == a_key.value {
                                for (b_key, b_value) in &b_object.values {
                                    if key.value == b_key.value {
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
                            if key.value == b_key.value {
                                // B
                                combined.push((key.clone(), b_value.clone()));
                                continue 'keys;
                            }
                        }
                    }

                    JsonType::Object(JsonObject {
                        values: combined,
                        position: a_object.position.clone()
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
                    JsonType::String(a_string.clone())
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
        let mut app = Cli::into_app();
        app.error(
            ErrorKind::ArgumentConflict,
            "You need to specify at least one folder",
        ).exit();
    }

    let files = read_folders(&args);
    let file_parity_success = args.skip_check_parity || check_file_parity(&files);
    if args.format {
        format_files(&args, &files);
    }
    let contents = read_files(&files);
    let file_style_success = args.skip_check_style || check_file_style(&contents);
    let jsons = parse_files(&contents);
    let key_order_success = args.skip_check_order || check_key_order(&jsons, &args.sort, &args.order);
    let key_parity_success = args.skip_check_parity || check_key_parity(&jsons);

    let success = file_parity_success && file_style_success && key_order_success && key_parity_success;

    if success {
        green_ln!("\nLint Successful");
    } else {
        red_ln!("\nLint Failed");
        exit(-1);
    }
}
