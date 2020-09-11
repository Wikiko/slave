use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
enum ReleaseTypes {
    MAJOR,
    MINOR,
    PATCH,
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    release_type: ReleaseTypes,
}

struct AndroidVersionUpgrader {
    file: File,
    current_version_code: u8,
    current_version_name: String,
}

impl AndroidVersionUpgrader {
    pub fn new() -> Result<AndroidVersionUpgrader, Box<dyn std::error::Error>> {
        let file = File::open("./build.gradle")?;
        let reader = BufReader::new(&file);
        let mut version_name = String::new();
        let mut version_code: u8 = 0;

        for line in reader.lines() {
            let line = line?;
            if line.contains("versionName") {
                version_name = String::from(line
                    .replace("\"", "")
                    .split_whitespace()
                    .last()
                    .expect("version name não encontrado"));
            }
            if line.contains("versionCode ") {
                version_code = line
                    .split_whitespace()
                    .last()
                    .expect("version code não encontrado")
                    .parse()
                    .expect("falha ao converter version code");
            }
        }

        Ok(AndroidVersionUpgrader {
            file,
            current_version_code: version_code,
            current_version_name: version_name,
        })
    }

    fn get_next_version_code(&self) -> u8 {
        self.current_version_code + 1
    }

    fn get_next_version_name(&self, release_type: ReleaseTypes) -> Result<String, Box<dyn std::error::Error>> {
        let splited_version_name = self.current_version_name.split(".");
        match release_type {
            ReleaseTypes::MAJOR => {
                Ok(String::new())
            }
            ReleaseTypes::MINOR => {
                Ok(String::new())
            }
            ReleaseTypes::PATCH => {
                let last = splited_version_name.last().unwrap();
                let last: u8 = last.parse()?;
                let last = last + 1;
                println!("{}", last);
                Ok(String::new())
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::from_args();
    let android_version_upgrader = AndroidVersionUpgrader::new()?;
    println!("version_code: {}", android_version_upgrader.current_version_code);
    println!("version_name: {}", android_version_upgrader.current_version_name);
    println!("nextVersioName: {:?}", android_version_upgrader.get_next_version_name(cli.release_type)?);

    Ok(())
}

fn increment_version_of_gradle() -> Result<(), io::Error> {
    let file = File::open("./build.gradle")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.contains("versionName") {
            let version_name = line
                .split_whitespace()
                .last()
                .expect("version name não encontrado");
            let version_name = version_name.replace("\"", "");
            let test = version_name.split(".");
            if let Some(last_version_number) = test.last() {
                let last_version_number: i32 = last_version_number
                    .parse()
                    .expect("Algo de errado não está certo");
            }
        }
    }

    Ok(())
}
