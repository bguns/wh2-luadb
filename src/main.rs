use clap::{load_yaml, App};
use std::path::Path;
use std::process::Command;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let mut rpfm_path = String::new();

    if let Some(rpfm) = matches.value_of("rpfm-path") {
        println!("Value for rpfm-path: {}", rpfm);
        rpfm_path = rpfm.to_owned();
    }

    if let Some(packfile) = matches.value_of("packfile") {
        let out_dir = if let Some(output_dir) = matches.value_of("output-dir") {
            output_dir.to_owned()
        } else {
            let packfile_dir = Path::new(packfile).parent().unwrap();
            let packfile_name = Path::new(packfile).file_stem().unwrap();
            packfile_dir
                .join(Path::new(&format!(
                    "{0}_lua_ext",
                    packfile_name.to_str().unwrap()
                )))
                .to_str()
                .unwrap()
                .to_owned()
        };

        println!("Packfile: {0}, out_dir: {1}", packfile, out_dir);
        println!("Running RPFM Command");

        Command::new(&rpfm_path)
            .args(&[
                "-g",
                "warhammer_2",
                "-p",
                &format!("{0}", packfile),
                "packfile",
                "-E",
                &format!("{0}", out_dir),
                "-",
                "db",
            ])
            .spawn()
            .expect("unable to run rpfm")
            .wait()
            .expect("error waiting for rpfm");
    }

    if let Some(directory) = matches.value_of("directory") {
        println!("Value for directory: {}", directory);
    }
}
