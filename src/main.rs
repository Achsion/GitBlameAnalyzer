mod config;

use std::{env, process};
use std::collections::HashMap;
use std::fs::metadata;
use std::path::PathBuf;
use std::process::Command;
use regex::Regex;
use crate::config::{AuthorAlias, Config};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!();
        eprintln!("\x1b[31;1mError: Missing config parameter!\x1b[0m");
        eprintln!("Usage: {} path/to/git/config.yml", &args[0]);
        process::exit(1);
    }

    let config_path = &args[1];
    let config_file = std::fs::File::open(config_path).expect("Could not open config file!");
    let config: Config = serde_yaml::from_reader(config_file).expect("Invalid config structure/values!");

    let project_dir = PathBuf::from(&config.project_dir);
    if metadata(&project_dir).is_err() {
        println!();
        eprintln!("\x1b[31;1mError: Could not find directory {}!\x1b[0m", &config.project_dir);
        process::exit(1);
    }

    let check_for_git = Command::new("git")
        .arg("-C")
        .arg(&config.project_dir)
        .arg("rev-parse")
        .output();
    if !check_for_git.unwrap().status.success() {
        eprintln!("\x1b[31;1mError: {} is not a git directory!\x1b[0m", &config.project_dir);
        process::exit(1);
    }

    let count_map = analyze_project(config.project_dir, config.author_mapping);

    println!("{:#?}", count_map);
}

fn analyze_project(project_dir: String, alias_mapping: Vec<AuthorAlias>) -> HashMap<String, u128> {
    let project_files = fetch_git_project_files(&project_dir);

    project_files.iter()
        .map(|project_file| blame_file(&project_dir, project_file))
        .map(|file_blame| count_blame_lines(file_blame, &alias_mapping))
        .reduce(|mut a, b| {
            b.iter().for_each(|(author, count)| *a.entry(String::from(&*author)).or_insert(0) += *count);
            a
        })
        .unwrap_or(HashMap::new())
}

fn count_blame_lines(file_blame: Vec<String>, _alias_mapping: &Vec<AuthorAlias>) -> HashMap<String, u128> {
    let blame_regex = Regex::new(r"[\^0-9a-zA-Z]{8}\s\S+\s+\((\S+)\s+\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\s+.\d{4}\s+\d+\)\s(.*)").unwrap();

    // TODO: filter out unwanted lines
    // TODO: map names

    file_blame.iter()
        .flat_map(|blame_line| {
            blame_regex
                .captures_iter(blame_line)
                .map(|c| c.extract())
                .map(|(_, [author, _line_content])| String::from(author))
        })
        .fold(HashMap::new(), |mut count_map, author| {
            *count_map.entry(author).or_insert(0) += 1;
            count_map
        })
}

fn blame_file(project_dir: &String, project_file: &String) -> Vec<String> {
    let git_blame = Command::new("git")
        .arg("-C")
        .arg(project_dir)
        .arg("blame")
        .arg(project_file)
        // .arg("-e") // uncomment this for mails
        .arg("-w")
        .arg("-f")
        .output();

    String::from_utf8(git_blame.unwrap().stdout)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

fn fetch_git_project_files(project_dir: &String) -> Vec<String> {
    let git_ls_tree = Command::new("git")
        .arg("-C")
        .arg(project_dir)
        .arg("ls-tree")
        .arg("--full-tree")
        .arg("-r")
        .arg("--name-only")
        .arg("HEAD")
        .output();

    String::from_utf8(git_ls_tree.unwrap().stdout)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}
