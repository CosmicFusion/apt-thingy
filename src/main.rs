use std::env;
use std::io::Read;
use std::process::exit;
use rust_apt::cache::*;
use rust_apt::new_cache;
use rust_apt::records::RecordField;
use rust_apt::*;
use std::process::{Command, Stdio};
use duct::cmd;
use regex::Regex;
use env::args;

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
    bin_pkg_name: String,
    #[arg(short, long)]
    /// the file to output to.
    file_path: String
}

fn main() {
    let args = Args::parse();

    println!("Hello {}!", args.operation);
}

fn get_rdepends_source_name() {
    let mut rdepends_vec = Vec::new();

    let mut srcbins_vec = Vec::new();

    // load apt cache
    let cache = new_cache!().unwrap();

    let apt_cache_command = match cmd!("apt-cache", "showsrc", "libgtk-4-dev").env("LANG", "en_US.UTF-8").stderr_capture().stdout_capture().run() {
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
    // Dedup to make get source names faster
    rdepends_vec.sort_unstable();
    rdepends_vec.dedup();
    for i in rdepends_vec.iter() {
        println!("{}", i)
    }
}
