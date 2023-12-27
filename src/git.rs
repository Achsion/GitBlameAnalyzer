use std::fs::metadata;
use std::path::PathBuf;
use std::process;
use std::process::Command;

pub struct GitRepository {
    pub id: String,
    pub project_dir: PathBuf,
}

impl GitRepository {
    pub fn from_directory(directory_path: PathBuf) -> Self {
        if metadata(&directory_path).is_err() {
            println!();
            eprintln!(
                "\x1b[31;1mError: Could not find directory {}!\x1b[0m",
                &directory_path.to_string_lossy()
            );
            process::exit(1); //TODO: dont exit process here - return Optional instead
        }

        let check_for_git = Command::new("git")
            .arg("-C")
            .arg(&directory_path)
            .arg("rev-parse")
            .output();
        if !check_for_git.unwrap().status.success() {
            eprintln!(
                "\x1b[31;1mError: {} is not a git directory!\x1b[0m",
                &directory_path.to_string_lossy()
            );
            process::exit(1);
        }

        let git_first_commit = Command::new("git")
            .arg("-C")
            .arg(&directory_path)
            .arg("rev-list")
            .arg("--parents")
            .arg("HEAD")
            .arg("|")
            .arg("egrep")
            .arg("\"^[a-f0-9]{40}$\"")
            .output()
            .unwrap();
        if git_first_commit.status.success() {
            eprintln!(
                "\x1b[31;1mError: {} does not have any commits!\x1b[0m",
                &directory_path.to_string_lossy()
            );
            process::exit(1);
        }

        let project_id = String::from_utf8(git_first_commit.stdout)
            .unwrap_or_default()
            .trim()
            .to_string();

        Self {
            id: project_id,
            project_dir: directory_path,
        }
    }
}
