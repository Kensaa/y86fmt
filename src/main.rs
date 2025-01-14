use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(help = "Path to the file to format", id = "filename", value_parser=parse_file)]
    filename: String,
    #[arg(long, help = "Flag to disable file backup")]
    disable_backup: bool,
}

fn parse_file(filename: &str) -> Result<String, String> {
    match File::open(filename) {
        Err(_) => Err(format!("file \"{}\" does not exists", filename)),
        Ok(_) => Ok(filename.to_string()),
    }
}

fn main() {
    let settings = Cli::parse();

    let file_path = std::path::Path::new(&settings.filename);
    let file_directory = file_path
        .parent()
        .expect("failed to get file parent directory");
    let filename = file_path
        .file_name()
        .expect("failed to get filename")
        .to_str()
        .unwrap()
        .to_string();

    if !settings.disable_backup {
        let mut backup_folder = file_directory.to_path_buf();
        backup_folder.push(".y86fmt-backup");
        if !backup_folder.exists() {
            fs::create_dir(&backup_folder).expect(&format!(
                "failed to create backup folder at {}",
                backup_folder.to_str().unwrap()
            ));
        }

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file_backup = backup_folder.join(format!("{}-{}", time, filename));
        fs::copy(&file_path, file_backup).expect("failed to make copy of file");
    }

    // We are sure that the file exists because of the value parser checking it before
    let file = File::open(&file_path).unwrap();
    let file = BufReader::new(file);
    let lines: Vec<String> = file
        .lines()
        .map(|line| line.expect("failed to read a line in the file"))
        .collect();
}
