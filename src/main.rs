use std::process::exit;
use rust_apt::new_cache;
use rust_apt::*;
use duct::cmd;
use regex::Regex;
use std::fs;
use std::path::Path;

use clap::Parser;

/// Apt thingy
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    /// get_rdepends or get_depends.
    operation: String,
    #[arg(short, long)]
    /// The package to run the opreation on.
    name: String,
    #[arg(short, long)]
    /// The package arch (amd64/i386/all)
    arch: String,
    #[arg(short, long)]
    /// the file to output to.
    file_path: String
}

fn main() {
    let args = Args::parse();

    match args.operation.as_str() {
        "get_rdepends" => {
            get_rdepends_source_name(args.name, args.arch, args.file_path)
        }
        "get_depends" => get_depends_source_name(args.name, args.arch, args.file_path),
        _ => {
            eprintln!("Error: --operation is invalid!");
            exit(2)
        }
    }
}

fn get_rdepends_source_name(bin_pkg_name: String, bin_pkg_arch: String, output_file_path: String) {
    let mut rdepends_vec = Vec::new();

    let mut srcbins_vec = Vec::new();

    // load apt cache
    let cache = new_cache!().unwrap();

    let apt_cache_command = match cmd!("apt-cache", "showsrc", bin_pkg_name).env("LANG", "en_US.UTF-8").stderr_capture().stdout_capture().run() {
        Ok(t) => {
            if String::from_utf8(t.stderr).unwrap().contains("Unable to locate package") {
                eprintln!("Selected Package has no Source Package entry");
                exit(6)
            } else {
                String::from_utf8(t.stdout).unwrap()
            }
        }
        _ => {
            eprintln!("Selected Package has no Source Package entry");
            exit(5)
        }
    };
    let result = apt_cache_command.lines()
        .filter(|&s| Regex::new("^Binary:").unwrap().is_match(&s))
        .collect::<Vec<_>>().concat();
    let mut result = result.replace("Binary: ", "");
    result.retain(|c| !c.is_whitespace());
    for bin_package in result.split(",") {
        srcbins_vec.push(bin_package)
    };
    srcbins_vec.sort_unstable();
    srcbins_vec.dedup();

    for src_bin in &srcbins_vec {
        // get record for requested package
        match bin_pkg_arch.as_str() {
            "all" => {
                match cache.get(src_bin) {
                    Some(pkg) => {
                        match pkg.rdepends().get(&DepType::Depends) {
                            Some(rdep_iter) => {
                                for dep in rdep_iter {
                                    match dep.first().target_package().candidate() {
                                        Some(t) => rdepends_vec.push(t.source_name().to_owned()),
                                        None => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                };
            }
            _ => {
                match cache.get(&(src_bin.to_string() + ":" + &bin_pkg_arch)) {
                    Some(pkg) => {
                        match pkg.rdepends().get(&DepType::Depends) {
                            Some(rdep_iter) => {
                                for dep in rdep_iter {
                                    match dep.first().target_package().candidate() {
                                        Some(t) => rdepends_vec.push(t.source_name().to_owned()),
                                        None => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                };
            }
        }
    }
    // Dedup to make get source names faster
    rdepends_vec.sort_unstable();
    rdepends_vec.dedup();

    let encoded_rdepends_vec = rdepends_vec
        .iter()
        .map(|pkg| format!("{pkg}"))
        .collect::<Vec<String>>()
        .join("\n");

    fs::remove_file(&output_file_path).expect("File already exists and can't be removed");
    fs::write(output_file_path, encoded_rdepends_vec).expect("Could not write file");

    for rdepend in rdepends_vec.iter() {
        println!("{}", rdepend)
    }
}

fn get_depends_source_name(bin_pkg_name: String, bin_pkg_arch: String, output_file_path: String) {
    let mut depends_vec = Vec::new();

    let mut srcbins_vec = Vec::new();

    // load apt cache
    let cache = new_cache!().unwrap();

    let apt_cache_command = match cmd!("apt-cache", "showsrc", bin_pkg_name).env("LANG", "en_US.UTF-8").stderr_capture().stdout_capture().run() {
        Ok(t) => {
            if String::from_utf8(t.stderr).unwrap().contains("Unable to locate package") {
                eprintln!("Selected Package has no Source Package entry");
                exit(6)
            } else {
                String::from_utf8(t.stdout).unwrap()
            }
        }
        _ => {
            eprintln!("Selected Package has no Source Package entry");
            exit(5)
        }
    };
    let result = apt_cache_command.lines()
        .filter(|&s| Regex::new("^Binary:").unwrap().is_match(&s))
        .collect::<Vec<_>>().concat();
    let mut result = result.replace("Binary: ", "");
    result.retain(|c| !c.is_whitespace());
    for bin_package in result.split(",") {
        srcbins_vec.push(bin_package)
    };
    srcbins_vec.sort_unstable();
    srcbins_vec.dedup();

    for src_bin in &srcbins_vec {
        match bin_pkg_arch.as_str() {
            "all" => {
                match cache.get(src_bin) {
                    Some(pkg) => {
                        match pkg.candidate().unwrap().depends_map().get(&DepType::Depends) {
                            Some(dep_iter) => {
                                for dep in dep_iter {
                                    match dep.first().target_package().candidate() {
                                        Some(t) => depends_vec.push(t.source_name().to_owned()),
                                        None => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                };
            }
            _ => {
                match cache.get(&(src_bin.to_string() + ":" + &bin_pkg_arch)) {
                    Some(pkg) => {
                        match pkg.candidate().unwrap().depends_map().get(&DepType::Depends) {
                            Some(dep_iter) => {
                                for dep in dep_iter {
                                    match dep.first().target_package().candidate() {
                                        Some(t) => depends_vec.push(t.source_name().to_owned()),
                                        None => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                };
            }
        }
    }

    // Dedup to make get source names faster
    depends_vec.sort_unstable();
    depends_vec.dedup();

    let encoded_depends_vec = depends_vec
        .iter()
        .map(|pkg| format!("{pkg}"))
        .collect::<Vec<String>>()
        .join("\n");
    if Path::new(&output_file_path).exists() {
        fs::remove_file(&output_file_path).expect("File already exists and can't be removed");
    }
    fs::write(output_file_path, encoded_depends_vec).expect("Could not write file");

    for depend in depends_vec.iter() {
        println!("{}", depend)
    }
}
