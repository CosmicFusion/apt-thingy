use std::process::exit;
use duct::cmd;
use regex::Regex;

use clap::Parser;

/// Apt thingy
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    /// The package to run the opreation on.
    name: String,
}

fn main() {
    let args = Args::parse();
    get_src_bins(args.name)
}

fn get_src_bins(bin_pkg_name: String) {

    let mut srcbins_vec = Vec::new();

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

    for bin_name in srcbins_vec.iter() {
        println!("{}", bin_name)
    }
}
