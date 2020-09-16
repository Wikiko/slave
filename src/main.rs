use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::fs::{File, remove_file, rename};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::process::{Command, exit};

use serde::Deserialize;
use structopt::StructOpt;

use ReleaseTypes::*;

#[derive(Deserialize, Debug)]
struct Config {
    develop_branch: String,
    android: String,
    package: String,
}

#[derive(StructOpt, Debug, PartialEq, Eq, Hash, Clone)]
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


struct SemanticVersion {
    current: HashMap<ReleaseTypes, u8>,
}

impl Clone for SemanticVersion {
    fn clone(&self) -> Self {
        let mut map = HashMap::new();
        map.clone_from(&self.current);
        SemanticVersion {
            current: map,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.current = source.current.clone();
    }
}

impl From<SemanticVersion> for String {
    fn from(version: SemanticVersion) -> Self {
        version.to_string()
    }
}

impl Display for SemanticVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl SemanticVersion {
    fn from_string(version_string: &String) -> SemanticVersion {
        let mut map: HashMap<ReleaseTypes, u8> = HashMap::new();
        let vec_version_string: Vec<&str> = version_string
            .split(".")
            .collect();

        map.insert(
            MAJOR, vec_version_string
                .get(0)
                .expect("Não foi possível pegar a MAJOR")
                .parse()
                .expect("Não foi possível pegar a MAJOR"),
        );
        map.insert(
            MINOR, vec_version_string
                .get(1)
                .expect("Não foi possível pegar a MAJOR")
                .parse()
                .expect("Não foi possível pegar a MAJOR"),
        );
        map.insert(
            PATCH, vec_version_string
                .get(2)
                .expect("Não foi possível pegar a MAJOR")
                .parse()
                .expect("Não foi possível pegar a MAJOR"),
        );

        SemanticVersion {
            current: map
        }
    }

    fn format_version_name(&self, vec: &[String; 3]) -> String {
        vec.join(".")
    }

    fn to_string(&self) -> String {
        let version_name_vec = [
            self.current[&MAJOR].to_string(),
            self.current[&MINOR].to_string(),
            self.current[&PATCH].to_string()
        ];
        self.format_version_name(&version_name_vec)
    }

    fn next_version(&self, release_type: &ReleaseTypes) -> SemanticVersion {
        match release_type {
            MAJOR => {
                let mut major = self.current[&MAJOR];
                major += 1;
                let new_version_name = [
                    major.to_string(),
                    String::from("0"),
                    String::from("0")
                ];

                SemanticVersion::from_string(
                    &self.format_version_name(&new_version_name)
                )
            }
            MINOR => {
                let mut minor = self.current[&MINOR];
                minor += 1;
                let new_version_name = [
                    self.current[&MAJOR].to_string(),
                    minor.to_string(),
                    String::from("0")
                ];
                SemanticVersion::from_string(
                    &self.format_version_name(&new_version_name)
                )
            }
            PATCH => {
                let mut patch = self.current[&PATCH];
                patch += 1;
                let new_version_name = [
                    self.current[&MAJOR].to_string(),
                    self.current[&MINOR].to_string(),
                    patch.to_string()
                ];
                SemanticVersion::from_string(
                    &self.format_version_name(&new_version_name)
                )
            }
        }
    }
}

struct AndroidVersionUpgrader {
    file_path: String,
    current_version_code: u8,
    current_version: SemanticVersion,
}

impl AndroidVersionUpgrader {
    pub fn new(gradle_path: &String, current_version: SemanticVersion) -> Result<AndroidVersionUpgrader, Box<dyn std::error::Error>> {
        let file = File::open(&gradle_path)?;
        let reader = BufReader::new(&file);
        let mut version_code: u8 = 0;

        for line in reader.lines() {
            let line = line?;
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
            file_path: gradle_path.clone(),
            current_version_code: version_code,
            current_version,
        })
    }

    fn get_next_version_code(&self) -> u8 {
        self.current_version_code + 1
    }

    fn get_current_version_name(&self) -> String {
        self.current_version.to_string()
    }

    fn get_next_version_name(&self, release_type: &ReleaseTypes) -> String {
        self.current_version.next_version(release_type).to_string()
    }

    fn upgrade(&self, release_type: &ReleaseTypes) -> Result<(), Box<dyn std::error::Error>> {
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
                        &self.get_next_version_code().to_string(),
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

struct PackageJsonUpgrader {
    file_path: String,
    current_version: SemanticVersion,
}

impl PackageJsonUpgrader {
    fn new(package_path: &String) -> Result<PackageJsonUpgrader, Box<dyn std::error::Error>> {
        let file = File::open(&package_path)?;
        let reader = BufReader::new(&file);
        let mut version_string = String::new();

        for line in reader.lines() {
            let line = line?;
            if line.contains("\"version\"") {
                version_string = String::from(line
                    .replace("\"", "")
                    .replace(",", "")
                    .split_whitespace()
                    .last()
                    .expect("version não encontrado"));
            }
        }

        Ok(PackageJsonUpgrader {
            file_path: package_path.clone(),
            current_version: SemanticVersion::from_string(&version_string),
        })
    }

    fn upgrade(&self, release_type: &ReleaseTypes) -> Result<(), Box<dyn std::error::Error>> {
        let new_file_path = format!("{}.new", &self.file_path);
        let file = File::open(&self.file_path)?;
        let new_file = File::create(&new_file_path)?;
        let reader = BufReader::new(&file);
        let mut writer = LineWriter::new(new_file);
        for line in reader.lines() {
            let line = line?;
            let mut to_write = line.clone();
            if line.contains("\"version\"") {
                let new_version_name = line
                    .replace(
                        &self.current_version.to_string(),
                        &self.current_version.next_version(release_type).to_string(),
                    );
                to_write = new_version_name;
            }
            writeln!(writer, "{}", to_write)?;
        }
        writer.flush()?;

        remove_file(&self.file_path)?;
        rename(&new_file_path, &self.file_path)?;

        Ok(())
    }
}

fn read_config() -> Config {
    let file_config = File::open("./slave.json")
        .expect("Não foi possível abrir o arquivo de configuração slave.json");
    serde_json::from_reader(file_config).expect("Formato do arquivo invalido!")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::from_args();

    println!("Lendo arquivo de configuração");
    let config = read_config();

    let package_upgrader = PackageJsonUpgrader::new(&config.package)?;

    let next_version = &package_upgrader
        .current_version.next_version(&cli.release_type);

    let android_version_upgrader = AndroidVersionUpgrader::new(
        &config.android,
        package_upgrader.current_version.clone()
    )?;

    println!("Criando a branch release/{}", &next_version);

    let git_checkout = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(format!("release/{}", &next_version))
        .arg(&config.develop_branch)
        .status()?;

    if !git_checkout.success() {
        exit(git_checkout.code().unwrap());
    }

    println!("Atualizando a versão no android");
    android_version_upgrader.upgrade(&cli.release_type)?;
    println!("Atualizando a versão no package.json");
    package_upgrader.upgrade(&cli.release_type)?;

    Ok(())
}
