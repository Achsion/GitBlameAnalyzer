mod config;

use crate::config::{AuthorAlias, Config, ProjectFileConfig};
use indicatif::ProgressBar;
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use std::fs::metadata;
use std::path::PathBuf;
use std::process::Command;
use std::{env, process};

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
    let config: Config =
        serde_yaml::from_reader(config_file).expect("Invalid config structure/values!");

    let project_dir = PathBuf::from(&config.project_dir);
    if metadata(project_dir).is_err() {
        println!();
        eprintln!(
            "\x1b[31;1mError: Could not find directory {}!\x1b[0m",
            &config.project_dir
        );
        process::exit(1);
    }

    let check_for_git = Command::new("git")
        .arg("-C")
        .arg(&config.project_dir)
        .arg("rev-parse")
        .output();
    if !check_for_git.unwrap().status.success() {
        eprintln!(
            "\x1b[31;1mError: {} is not a git directory!\x1b[0m",
            &config.project_dir
        );
        process::exit(1);
    }

    let count_map = analyze_project(config);
    output_result(count_map);
}

fn output_result(count_map: HashMap<String, u128>) {
    let mut hash_vec: Vec<(&String, &u128)> = count_map.iter().collect();

    hash_vec.sort_by(|(_author1, count1), (_author2, count2)| count2.cmp(count1));

    let out = hash_vec
        .iter()
        .map(|(author, count)| format!("- {}: {}", author, count))
        .join("\n");

    println!("\n\nLines of code per developer:\n{}", out);
}

fn analyze_project(config: Config) -> HashMap<String, u128> {
    let project_files = fetch_git_project_files(&config.project_dir, &config.project_files);

    let progress_bar = ProgressBar::new(project_files.len() as u64);

    let count_map = project_files
        .iter()
        .map(|project_file| blame_file(&config.project_dir, project_file))
        .map(|file_blame| count_blame_lines(file_blame, &config.author_mapping))
        .reduce(|mut a, b| {
            b.iter().for_each(|(author, count)| {
                *a.entry(String::from(&*author)).or_insert(0) += *count
            });
            progress_bar.inc(1);
            a
        })
        .unwrap_or(HashMap::new());

    progress_bar.finish_with_message("done");

    count_map
}

fn count_blame_lines(
    file_blame: Vec<String>,
    alias_mapping: &[AuthorAlias],
) -> HashMap<String, u128> {
    let blame_regex = Regex::new(r"[\^0-9a-zA-Z]{8}\s\S*\s*\((.+)\s+\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\s+.\d{4}\s+\d+\)\s(.*)").unwrap();

    file_blame
        .iter()
        .flat_map(|blame_line| {
            blame_regex
                .captures_iter(blame_line)
                .map(|c| c.extract())
                .filter(|(_, [_author, line_content])| !line_content.trim().is_empty())
                .map(|(_, [author, _line_content])| String::from(author.trim()))
                .map(|author| map_author(author, alias_mapping))
        })
        .fold(HashMap::new(), |mut count_map, author| {
            *count_map.entry(author).or_insert(0) += 1;
            count_map
        })
}

fn map_author(author: String, alias_mapping: &[AuthorAlias]) -> String {
    alias_mapping
        .iter()
        .find(|alias| alias.author == author)
        .map(|alias| String::from(&*alias.map_to))
        .unwrap_or(author)
}

fn blame_file(project_dir: &String, project_file: &String) -> Vec<String> {
    let git_blame = Command::new("git")
        .arg("-C")
        .arg(project_dir)
        .arg("blame")
        .arg(project_file)
        // .arg("-e") // uncomment this to display mail address of authors !!! the regex needs to be changed for this, i'm just too lazy to do it right now !!!
        .arg("-w")
        .arg("-f")
        .output();

    String::from_utf8(git_blame.unwrap().stdout)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect()
}

fn fetch_git_project_files(
    project_dir: &String,
    project_file_config: &ProjectFileConfig,
) -> Vec<String> {
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
        .filter(|file| {
            !project_file_config
                .blacklist
                .iter()
                .any(|regex| regex.is_match(file))
        })
        .collect()
}
