//! simple cat like
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(clippy::expect_fun_call)]

extern crate clap;

use clap::{Command, Arg};
use std::path::Path;
use std::process;
use std::fs;
use std::io;

use std::io::BufRead;
use std::io::Write;

fn main () {
    let args = Command::new("kt")
        .version("0.0.1")
        .author("JuPengfei")
        .about("simple cmdline like cat")
        .arg(Arg::new("FILE")
                .help("File to print")
                .required(true))
        .get_matches();

    if let Some(c) = args.get_one::<String>("FILE") {
        if Path::new(c.as_str()).is_file() {
            match fs::File::open(c) {
                Ok(f) => {
                    let in_buf = io::BufReader::new(f);
                    let mut out_buf = io::BufWriter::new(io::stdout());

                    for content in in_buf.lines() {
                        match content {
                            Ok(cont) => {
                                if writeln!(out_buf, "{}", cont).is_err() {
                                    eprintln!("write to stdout error: {}", cont);
                                    process::exit(1);
                                }
                            }
                            Err(e) => {
                                eprintln!("read file error: {}", e);
                                process::exit(1);
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Can't open file or directory: {}", c);
                    process::exit(1);
                }
            }
        }
        else {
            eprintln!("No such file or directory: {}", c);
            process::exit(1);
        }
    }
}