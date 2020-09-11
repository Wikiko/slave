use std::process::Command;
use std::io::{stderr, Write, BufReader, BufRead};
use std::fs::File;
use std::io;

fn main() {
    let git_command = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg("release/1.0.2")
        .output()
        .expect("Git failed to execute command");

    if !git_command.status.success() {
        println!("Não foi possível trocar de branch");
        stderr().write_all(&git_command.stderr).expect("Falha ao reportar o resultado do comando git");
        return;
    }
    stderr().write_all(&git_command.stderr).expect("Falha ao reportar o resultado do comando git");

    increment_version_of_gradle().expect("Ocorreu um erro ao trocar a versão no gradle");

}

fn increment_version_of_gradle() -> Result<(), io::Error> {
    let file = File::open("./build.gradle")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.contains("versionName") {
            let version_name = line.split_whitespace().last().expect("version name não encontrado");
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
