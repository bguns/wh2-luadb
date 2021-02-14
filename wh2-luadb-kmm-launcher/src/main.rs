use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;

use winreg::enums::*;
use winreg::RegKey;

use crossterm::event::read;

fn main() -> Result<(), Wh2LuaDBKMMLauncherError> {
    let mut step = 0;
    let wh2_install_dir = find_wh2_install_dir()?;
    let result = do_the_things(&mut step, &wh2_install_dir);
    match result {
        Err(ref error) => {
            eprintln!("{}", error);
            let mut wh2_exe = wh2_install_dir.clone();
            wh2_exe.push("Warhammer2.exe");
            let mut wh2_real_exe = wh2_install_dir.clone();
            wh2_real_exe.push("Warhammer2_real.exe");
            let mut wh2_luadb_exe = wh2_install_dir.clone();
            wh2_luadb_exe.push("wh2-luadb.exe");
            if step == 1 {
                fs::rename(&wh2_real_exe, &wh2_exe)?;
            } else if step == 2 || step == 3 {
                fs::rename(&wh2_exe, &wh2_luadb_exe)?;
                fs::rename(&wh2_real_exe, &wh2_exe)?;
            } else if step == 4 {
                fs::rename(&wh2_real_exe, &wh2_exe)?;
            }
            eprintln!("Press any key to quit");
            match read().unwrap() {
                _ => {}
            }
            Ok(())
        }
        Ok(()) => Ok(()),
    }
}

fn do_the_things(
    step: &mut i32,
    wh2_install_dir: &PathBuf,
) -> Result<(), Wh2LuaDBKMMLauncherError> {
    println!("WH2 install dir found at {}", wh2_install_dir.display());
    let mut wh2_exe = wh2_install_dir.clone();
    wh2_exe.push("Warhammer2.exe");
    let mut wh2_real_exe = wh2_install_dir.clone();
    wh2_real_exe.push("Warhammer2_real.exe");
    let mut wh2_luadb_exe = wh2_install_dir.clone();
    wh2_luadb_exe.push("wh2-luadb.exe");

    let wh2_luadb_local_exe = PathBuf::from("./wh2-luadb.exe");
    if !wh2_luadb_local_exe.exists() {
        return Err(Wh2LuaDBKMMLauncherError::Error("Could not find wh2-luadb.exe in this directory. Please make sure you run this executable in the same directory as Warhammer2MM.exe and wh2-luadb.exe.".to_string()));
    }

    let kmm_local_exe = PathBuf::from("./Warhammer2MM.exe");
    if !kmm_local_exe.exists() {
        return Err(Wh2LuaDBKMMLauncherError::Error("Could not find Warhammer2MM.exe in this directory. Please make sure you run this executable in the same directory as Warhammer2MM.exe and wh2-luadb.exe.".to_string()));
    }

    fs::copy(&wh2_luadb_local_exe, &wh2_luadb_exe)?;

    fs::rename(&wh2_exe, &wh2_real_exe)?;
    *step += 1;
    fs::rename(&wh2_luadb_exe, &wh2_exe)?;
    *step += 1;
    println!("Starting KMM. DO NOT CLOSE THIS WINDOW.");
    Command::new("./Warhammer2MM.exe").output()?;
    *step += 1;
    fs::rename(&wh2_exe, &wh2_luadb_exe)?;
    *step += 1;
    fs::rename(&wh2_real_exe, &wh2_exe)?;
    *step += 1;
    Ok(())
}

fn find_wh2_install_dir() -> Result<PathBuf, Wh2LuaDBKMMLauncherError> {
    let hkcr = RegKey::predef(HKEY_CURRENT_USER);
    let steam_path: String = hkcr
        .open_subkey("SOFTWARE\\Valve\\Steam")?
        .get_value("SteamPath")?;
    println!("Steam path found at {}", steam_path);
    let mut wh2_path = PathBuf::from(&steam_path);
    wh2_path.push("steamapps");
    wh2_path.push("common");
    wh2_path.push("Total War WARHAMMER II");
    if !wh2_path.exists() {
        println!(
            "Warhammer 2 install not found at {}.\nLooking for alternate steam libraries...",
            wh2_path.display()
        );
        let mut libraryfolders_path = PathBuf::from(&steam_path);
        libraryfolders_path.push("steamapps");
        libraryfolders_path.push("libraryfolders.vdf");
        if !libraryfolders_path.exists() {
            return Err(Wh2LuaDBKMMLauncherError::Error(
                "Could not find Warhammer2 install directory".to_string(),
            ));
        }

        let file = fs::File::open(libraryfolders_path)?;
        let buf = BufReader::new(file);
        let mut searching = false;
        for line in buf.lines().map(|l| l.unwrap()) {
            if line.trim().starts_with("\"1\"") {
                searching = true;
            }
            if searching && line.trim().starts_with("}") {
                searching = false;
            }

            if searching {
                let start_path = &line
                    .trim()
                    .split("\t")
                    .collect::<Vec<&str>>()
                    .pop()
                    .unwrap();
                wh2_path =
                    PathBuf::from(start_path.trim()[1..start_path.len() - 1].replace("\\\\", "\\"));
                wh2_path.push("steamapps");
                wh2_path.push("common");
                wh2_path.push("Total War WARHAMMER II");
                if !wh2_path.exists() {
                    println!("Warhammer 2 install not found at {}", wh2_path.display());
                } else {
                    break;
                }
            }
        }
    }
    if !wh2_path.exists() {
        return Err(Wh2LuaDBKMMLauncherError::Error(
            "Could not find Warhammer2 install directory".to_string(),
        ));
    }

    Ok(wh2_path)
}

#[derive(Debug)]
enum Wh2LuaDBKMMLauncherError {
    Error(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for Wh2LuaDBKMMLauncherError {
    fn from(err: std::io::Error) -> Self {
        Wh2LuaDBKMMLauncherError::IoError(err)
    }
}

impl fmt::Display for Wh2LuaDBKMMLauncherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", "[ERROR]")?;
        match &self {
            &Wh2LuaDBKMMLauncherError::Error(message) => {
                write!(f, "{}", message)
            }
            &Wh2LuaDBKMMLauncherError::IoError(io_error) => {
                write!(f, "Unexpected IO error: {}", io_error)
            }
        }
    }
}
