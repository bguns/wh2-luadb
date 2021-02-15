use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;

use std::sync::{Arc, RwLock};

use winreg::enums::*;
use winreg::RegKey;

use winapi::shared::minwindef::{BOOL, DWORD, FALSE, TRUE};
use winapi::um::consoleapi::SetConsoleCtrlHandler;
use winapi::um::wincon::{
    CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
};

use lazy_static::lazy_static;

use crossterm::event::read;

lazy_static! {
    static ref WH2_DIR: Arc<RwLock<PathBuf>> = Arc::new(RwLock::new(
        find_wh2_install_dir().expect("Unexpected error when trying to find WH2 DIR")
    ));
    static ref WH2_EXE: Arc<PathBuf> = {
        let mut wh2_exe = WH2_DIR.read().unwrap().clone();
        wh2_exe.push("Warhammer2.exe");
        Arc::new(wh2_exe)
    };
    static ref WH2_REAL_EXE: Arc<PathBuf> = {
        let mut wh2_real_exe = WH2_DIR.read().unwrap().clone();
        wh2_real_exe.push("Warhammer2_real.exe");
        Arc::new(wh2_real_exe)
    };
    static ref WH2_LUADB_EXE: Arc<PathBuf> = {
        let mut wh2_luadb_exe = WH2_DIR.read().unwrap().clone();
        wh2_luadb_exe.push("wh2-luadb.exe");
        Arc::new(wh2_luadb_exe)
    };
    static ref STEP_REACHED: Arc<RwLock<i32>> = Arc::new(RwLock::new(0));
}

fn main() -> Result<(), Wh2LuaDBKMMLauncherError> {
    if 0 == unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) } {
        println!("Could not install control handler");
        eprintln!("Press any key to quit");
        match read().unwrap() {
            _ => {}
        }
        return Ok(());
    }

    let result = do_the_things();
    match result {
        Err(ref error) => {
            eprintln!("{}", error);
            handle_interrupted();
            eprintln!("Press any key to quit");
            match read().unwrap() {
                _ => {}
            }
            Ok(())
        }
        Ok(()) => Ok(()),
    }
}

fn handle_interrupted() {
    let step = *STEP_REACHED.read().unwrap();
    if step == 1 {
        println!("Restoring Warhammer2.exe from Warhammer2.exe");
        fs::rename(&**WH2_REAL_EXE, &**WH2_EXE)
            .expect("Critical error when restoring files following interruption");
    } else if step == 2 || step == 3 {
        println!("Restoring wh2-luadb.exe from Warhammer2.exe");
        fs::rename(&**WH2_EXE, &**WH2_LUADB_EXE)
            .expect("Critical error when restoring files following interruption");
        println!("Restoring Warhammer2.exe from Warhammer2.exe");
        fs::rename(&**WH2_REAL_EXE, &**WH2_EXE)
            .expect("Critical error when restoring files following interruption");
    } else if step == 4 {
        println!("Restoring Warhammer2.exe from Warhammer2.exe");
        fs::rename(&**WH2_REAL_EXE, &**WH2_EXE)
            .expect("Critical error when restoring files following interruption");
    }
}

fn do_the_things() -> Result<(), Wh2LuaDBKMMLauncherError> {
    println!(
        "Warhammer 2 install directory found at {}",
        WH2_DIR.read().unwrap().display()
    );

    let wh2_luadb_local_exe = PathBuf::from("./wh2-luadb.exe");
    if !wh2_luadb_local_exe.exists() {
        return Err(Wh2LuaDBKMMLauncherError::Error("Could not find wh2-luadb.exe in this directory. Please make sure you run this executable in the same directory as Warhammer2MM.exe and wh2-luadb.exe.".to_string()));
    }

    let kmm_local_exe = PathBuf::from("./Warhammer2MM.exe");
    if !kmm_local_exe.exists() {
        return Err(Wh2LuaDBKMMLauncherError::Error("Could not find Warhammer2MM.exe in this directory. Please make sure you run this executable in the same directory as Warhammer2MM.exe and wh2-luadb.exe.".to_string()));
    }

    println!("Copying wh2-luadb.exe to Warhammer 2 install directory...");
    fs::copy(&wh2_luadb_local_exe, &**WH2_LUADB_EXE)?;

    println!("Renaming Warhammer2.exe to Warhammer2_real.exe (will be restored when KMM closes)");
    fs::rename(&**WH2_EXE, &**WH2_REAL_EXE)?;
    *STEP_REACHED.write().unwrap() += 1;

    println!("Renaming wh2-luadb.exe to Warhammer2.exe (will be restored when KMM closes)");
    fs::rename(&**WH2_LUADB_EXE, &**WH2_EXE)?;
    *STEP_REACHED.write().unwrap() += 1;

    println!("Starting KMM...");
    Command::new("./Warhammer2MM.exe").output()?;
    *STEP_REACHED.write().unwrap() += 1;

    println!("Restoring wh2-luadb.exe from Warhammer2.exe");
    fs::rename(&**WH2_EXE, &**WH2_LUADB_EXE)?;
    *STEP_REACHED.write().unwrap() += 1;

    println!("Restoring Warhammer2.exe from Warhammer2_real.exe");
    fs::rename(&**WH2_REAL_EXE, &**WH2_EXE)?;
    *STEP_REACHED.write().unwrap() += 1;

    Ok(())
}

fn find_wh2_install_dir() -> Result<PathBuf, Wh2LuaDBKMMLauncherError> {
    // Get the steam install dir from windows registry
    let hkcr = RegKey::predef(HKEY_CURRENT_USER);
    let steam_path: String = hkcr
        .open_subkey("SOFTWARE\\Valve\\Steam")?
        .get_value("SteamPath")?;
    println!("Steam path found at {}", steam_path);
    // Look in the default steamapps directory for WH2
    let mut wh2_path = PathBuf::from(&steam_path);
    wh2_path.push("steamapps");
    wh2_path.push("common");
    wh2_path.push("Total War WARHAMMER II");
    // If it's not there, we search for alternate library locations
    if !wh2_path.exists() {
        println!(
            "Warhammer 2 install not found at {}.\nLooking for alternate steam libraries...",
            wh2_path.display()
        );
        // Steam stores alternate library locations in a file called libraryfolders.vdf
        let mut libraryfolders_path = PathBuf::from(&steam_path);
        libraryfolders_path.push("steamapps");
        libraryfolders_path.push("libraryfolders.vdf");
        if !libraryfolders_path.exists() {
            return Err(Wh2LuaDBKMMLauncherError::Error(
                "Could not find Warhammer2 install directory".to_string(),
            ));
        }

        // The libraryfolders.vdf file is a text file with a dumb format.
        // We search for lines of the form "<number>" <tab> "Path\\To\\SteamLibrary",
        // then split, ignore the quotes and replace the double backslashes by singles.
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

extern "system" fn ctrl_handler(ctrl_type: DWORD) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT => {
            eprintln!("Break detected. Restoring files...");
            handle_interrupted();
            FALSE
        }
        CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT => {
            eprintln!("Close window detected. Restoring files...");
            handle_interrupted();
            TRUE
        }
        _ => FALSE,
    }
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
