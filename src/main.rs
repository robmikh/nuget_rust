use std::{process::Command, path::{Path, PathBuf}, fs::File, io::Read};

use clap::Parser;
use cli::{Args, Platform};

mod cli;

fn main() {
    let args = Args::parse();

    let all = args.all;
    let pack = all || args.pack;

    let (build, targets) = {
        let mut targets = Vec::with_capacity(args.build.len());
        for platform in &args.build {
            targets.push(platform.to_rust_target().to_owned());
        }
        if all && targets.is_empty() {
            let build_platforms = [Platform::x64, Platform::ARM64];
            for platform in build_platforms {
                targets.push(platform.to_rust_target().to_owned());
            }
        }
        (all || !args.build.is_empty(), targets)
    };

    if let Some(current_dir) = args.dir.as_ref() {
        println!("Using {}...", current_dir);
        std::env::set_current_dir(current_dir).expect("Failed to set the current directory.");
    }

    if build {
        println!("Building...");
        for target in &targets {
            println!("  {}...", target);
            build_target(target);
        }
    }

    if pack {
        println!("Packing...");
        nuget_pack();
    }
}

fn build_target(target: &str) {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg(target)
        .status()
        .expect("process failed to execute");

    if !status.success() {
        panic!("Failed to build '{}' target!", target);
    }
}

fn nuget_pack() {
    let nuget_path = Path::new("nuget");
    if nuget_path.exists() {
        nuget_pack_directory(nuget_path);
    } else {
        panic!("No nuget directory found!");
    }
}

fn nuget_pack_directory<P: AsRef<Path>>(nuget_path: P) {
    let nuspec = get_nugetpkg_nuspec(&nuget_path);
    let version = get_nugetpkg_version(nuget_path);

    let status = Command::new("nuget")
        .arg("pack")
        .arg(nuspec)
        .args(&["-version", &version])
        .status()
        .expect("process failed to execute");
    if !status.success() {
        panic!("nuget pack failed!");
    }
}

fn get_nugetpkg_nuspec<P: AsRef<Path>>(nuget_path: P) -> std::path::PathBuf {
    let mut nuspec_paths =
        get_files_with_extension(nuget_path, "nuspec").expect("Failed to look for solution files.");

    if nuspec_paths.is_empty() {
        panic!("No nuspec files found!");
    } else if nuspec_paths.len() > 1 {
        panic!("Too many nuspec files found!");
    }

    nuspec_paths.pop().unwrap()
}

fn get_nugetpkg_version<P: AsRef<Path>>(nuget_path: P) -> String {
    let version_path = {
        let mut version_path = nuget_path.as_ref().to_owned();
        version_path.push("VERSION");
        version_path
    };
    let mut version_file = File::open(version_path).expect("Failed to open VERSION file");

    let mut version_string = String::new();
    version_file.read_to_string(&mut version_string).unwrap();

    version_string
}

fn get_files_with_extension<P: AsRef<Path>>(folder_path: P, ext: &str) -> Option<Vec<PathBuf>> {
    let folder_path = folder_path.as_ref();
    let file_paths = std::fs::read_dir(folder_path).ok()?;
    let mut paths = Vec::new();
    for entry in file_paths {
        if let Ok(entry) = entry {
            let file_path = entry.path();
            if let Some(file_ext) = file_path.extension() {
                if file_ext == ext {
                    paths.push(file_path);
                }
            }
        }
    }
    Some(paths)
}
