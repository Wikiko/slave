use std::collections::HashMap;
use std::fs::{File, rename, remove_file};
use std::io::{BufRead, BufReader, LineWriter, Write};

use structopt::StructOpt;

use ReleaseTypes::*;
use std::process::Command;

#[derive(StructOpt, Debug, PartialEq, Eq, Hash)]
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
    file_path: String,
    current_version_code: u8,
    current_version_name: HashMap<ReleaseTypes, u8>,
}

impl AndroidVersionUpgrader {
    pub fn new() -> Result<AndroidVersionUpgrader, Box<dyn std::error::Error>> {
        let file_path = String::from("./build.gradle");
        let file = File::open(&file_path)?;
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

        let mut map: HashMap<ReleaseTypes, u8> = HashMap::new();
        let splited_version_name: Vec<&str> = version_name.split(".").collect();
        map.insert(MAJOR, splited_version_name[0]
            .parse()
            .expect("Não foi possível pegar a versão MAJOR"));
        map.insert(MINOR, splited_version_name[1]
            .parse()
            .expect("Não foi possível pegar a versão MINOR"));
        map.insert(PATCH, splited_version_name[2]
            .parse()
            .expect("Não foi possível pegar a versão PATCH"));

        let map = map;

        Ok(AndroidVersionUpgrader {
            file_path,
            current_version_code: version_code,
            current_version_name: map,
        })
    }

    fn get_next_version_code(&self) -> u8 {
        self.current_version_code + 1
    }

    fn get_current_version_name(&self) -> String {
        let version_name_vec = [
            self.current_version_name[&MAJOR].to_string(),
            self.current_version_name[&MINOR].to_string(),
            self.current_version_name[&PATCH].to_string()
        ];
        self.format_version_name(&version_name_vec)
    }

    fn get_next_version_name(&self, release_type: &ReleaseTypes) -> String {
        match release_type {
            MAJOR => {
                let mut major = self.current_version_name[&MAJOR];
                major += 1;
                let new_version_name = [
                    major.to_string(),
                    String::from("0"),
                    String::from("0")
                ];
                self.format_version_name(&new_version_name)
            }
            MINOR => {
                let mut minor = self.current_version_name[&MINOR];
                minor += 1;
                let new_version_name = [
                    self.current_version_name[&MAJOR].to_string(),
                    minor.to_string(),
                    String::from("0")
                ];
                self.format_version_name(&new_version_name)
            }
            PATCH => {
                let mut patch = self.current_version_name[&PATCH];
                patch += 1;
                let new_version_name = [
                    self.current_version_name[&MAJOR].to_string(),
                    self.current_version_name[&MINOR].to_string(),
                    patch.to_string()
                ];
                self.format_version_name(&new_version_name)
            }
        }
    }

    fn format_version_name(&self, vec: &[String; 3]) -> String {
        vec.join(".")
    }

    fn upgrade(&self, release_type: ReleaseTypes) -> Result<(), Box<dyn std::error::Error>> {
        let new_file_path = format!("{}.new", &self.file_path);
        let file = File::open(&self.file_path)?;
        let new_file = File::create(&new_file_path)?;
        let reader = BufReader::new(&file);
        let mut writer = LineWriter::new(new_file);
        for line in reader.lines() {
            let line = line?;
            let mut to_write = line.clone();
            if line.contains("versionName") {
                let new_version_name = line
                    .replace(
                        &self.get_current_version_name(),
                        &self.get_next_version_name(&release_type),
                    );
                to_write = new_version_name;
            }
            if line.contains("versionCode ") {
                let new_version_code = line
                    .replace(
                        &self.current_version_code.to_string(),
                        &self.get_next_version_code().to_string()
                    );
                to_write = new_version_code;
            }
            writeln!(writer, "{}", to_write)?;
        }
        writer.flush()?;

        remove_file(&self.file_path)?;
        rename(&new_file_path, &self.file_path)?;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::from_args();

    let android_version_upgrader = AndroidVersionUpgrader::new()?;


    Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg("master")
        .arg(format!("release/{}", android_version_upgrader.get_next_version_name(&cli.release_type)))
        .spawn()?;

    android_version_upgrader.upgrade(cli.release_type)?;

    Ok(())
}
