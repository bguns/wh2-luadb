use clap::{load_yaml, App, ArgMatches};
use std::path::PathBuf;
use std::process::Command;

struct Config {
    rpfm_path: PathBuf,
    packfile: Option<PathBuf>,
    _in_dir: Option<PathBuf>,
    out_dir: PathBuf,
}

impl Config {
    fn from_matches(matches: &ArgMatches) -> Config {
        let rpfm_path: PathBuf = if let Some(rpfm) = matches.value_of("rpfm-path") {
            PathBuf::from(rpfm)
        } else {
            println!("[ERROR] Please provide a path to rpfm_cli.exe");
            panic!();
        };

        if !rpfm_path.exists() {
            println!("[ERROR] path to RPFM cli not found");
            panic!();
        }

        let packfile_path = if let Some(packfile) = matches.value_of("packfile") {
            Some(PathBuf::from(packfile))
        } else {
            None
        };

        let in_dir_path = if let Some(directory) = matches.value_of("directory") {
            Some(PathBuf::from(directory))
        } else {
            None
        };

        let out_dir_path = if let Some(output_dir) = matches.value_of("output-dir") {
            PathBuf::from(output_dir)
        } else {
            if let Some(ref packfile) = packfile_path {
                let packfile_dir = packfile.parent().unwrap();
                let packfile_name = packfile.file_stem().unwrap();
                let mut dir = PathBuf::from(packfile_dir);
                dir.push(&format!("{0}_lua_ext", packfile_name.to_str().unwrap()));
                dir
            } else {
                PathBuf::new()
            }
        };

        Config {
            rpfm_path,
            packfile: packfile_path,
            _in_dir: in_dir_path,
            out_dir: out_dir_path,
        }
    }
}

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let config = Config::from_matches(&matches);

    println!("Running RPFM Command");

    if let Some(ref packfile) = config.packfile {
        Command::new(&config.rpfm_path)
            .args(&[
                "-g",
                "warhammer_2",
                "-p",
                &format!("{0}", &packfile.to_str().unwrap()),
                "packfile",
                "-E",
                &format!("{0}", &config.out_dir.to_str().unwrap()),
                "-",
                "db",
            ])
            .spawn()
            .expect("unable to run rpfm")
            .wait()
            .expect("error waiting for rpfm");
    }
}
