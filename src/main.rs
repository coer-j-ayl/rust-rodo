//! rust todo
#![allow(unused_macros)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::needless_return)]

extern crate logger;
extern crate log;

use std::fs::{File, OpenOptions};
use std::io::{Result, Error, ErrorKind, Seek, Write, BufRead};
use std::io;
use std::fmt;
use std::env;
use std::convert;

// constant
const RODO_LOG_TAG: &str = "rodo";
const RODO_VERSION: &str = "1.0.0";
const RODO_DB_PATH: &str = "/sdcard/todo.db";

/// ===============> Log <===================
pub fn init_logger () {
    let mut config = logger::Config::default();

    config = config.with_min_level(log::Level::Trace);
    config = config.with_tag_on_device(RODO_LOG_TAG);

    logger::init(config);
}

macro_rules! LOGV {
    ($($arg:tt)+) => ({
        log::trace!($($arg)+);
        println!($($arg)+);
    });
}

macro_rules! LOGD {
    ($(&arg:tt)+) => ({
        log::debug!($($arg)+);
        println!($($arg)+);
    })
}

macro_rules! LOGI {
    ($($arg:tt)+) => ({
        log::info!($($arg)+);
        println!($($arg)+);
    })
}

macro_rules! LOGW {
    ($($arg:tt)+) => ({
        log::warn!($($arg)+);
        println!($($arg)+);
    })
}

macro_rules! LOGE {
    ($($arg:tt)+) => ({
        log::error!($($arg)+);
        println!($($arg)+);
    })
}

/// ==============> TodoDB <==================
/// todo item
struct Record {
    sequence: usize,
    content: String,
}

impl Record {
    fn new (seq: usize, msg: &str) -> Record {
        Record {sequence: seq, content: msg.to_string()}
    }

    fn from (field: &str) -> Option<Record> {
        let sf: Vec<&str> = field.split(' ').collect();
        if sf.len() != 2_usize {
            return None;
        }

        let seq = sf[0].parse::<usize>();
        if seq.is_err() {
            return None;
        }

        Some(Record {
            sequence: seq.unwrap(),
            content: sf[1].to_string(),
        })
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.sequence, self.content)
    }
}

struct TodoDB {
    file_path: String,
    file_handle: File,
    file_index: usize,
}

impl TodoDB {
    fn new (path: impl convert::Into<String>) -> Result<TodoDB> {
        let path_str = path.into();
        let f = OpenOptions::new().create(true).read(true).write(true).append(true).open(&path_str)?;
        let mut db = TodoDB {file_path: path_str, file_handle: f, file_index: 0};

        let records = db.query();
        let max_record = records.iter().max_by_key(|x| x.sequence);
        if let Some(r) = max_record {
            db.file_index = r.sequence + 1;
        }

        Ok(db)
    }

    fn add_record (&mut self, content: &str) -> Result<()> {
        self.file_handle.seek(io::SeekFrom::End(0))?;

        let r = Record::new(self.file_index, content);
        self.file_index += 1;

        match self.file_handle.write(format!("{}\n", r).as_bytes()) {
            Ok(_) => Ok(()),
            Err(n) => Err(n), 
        }
    }

    fn del_record (&mut self, sequence: &str) -> Result<()> {
        let mut records = self.query();
        let seq = match sequence.parse::<usize>() {
            Err(n) => Err(Error::new(ErrorKind::Other, n.to_string()))?,
            Ok(n) => n,
        };

        let index = records.iter().position(|x| x.sequence == seq);
        if let Some(i) = index {
            records.remove(i);
        }
        else {
            return Err(Error::new(ErrorKind::Other, "not valid sequence"));
        }

        self.write_record(&records)?;
        Ok(())
    }

    fn write_record (&mut self, rds: &[Record]) -> Result<()> {
        self.file_handle.set_len(0)?;
        for r in rds {
            if let Err(e) = self.file_handle.write(format!("{}\n", r).as_bytes()) {
                LOGE!("write record [{}] error: {}", r, e);
            }
        }

        Ok(())
    }

    fn query (&mut self) -> Vec<Record> {
        let mut records: Vec<Record> = vec![];
        if let Err(e) = self.file_handle.rewind() {
            LOGE!("query rewind error: {}", e);
            return records;
        }

        let buf_reader = io::BufReader::new(&self.file_handle);
        for line in buf_reader.lines() {
            if let Err(e) = line {
                LOGE!("query error: {}", e);
                break;
            }

            let rd = Record::from(&line.unwrap());
            if let Option::Some(r) = rd {
                records.push(r);
            }
        }

        records
    }
}

/// ===============> MAIN <===================
enum RodoCommand {
    None(String),
    List,
    Info,
    Help,
    Version,
    Add(String),
    Remove(String),
}

fn show_help () {
    LOGI!("rodo {}", RODO_VERSION);
    LOGI!(r#"
USAGE:
    rodo <option|subcommand> 

OPTIONALS:
    -h, --help     Print help information
    -v, --version  Print version information

SUBCOMMANDS:
    add      Add a todo item
    list     List all todo items
    remove   Remove a todo item
    info     Show todo info
    "#);
}

fn parse_command () -> RodoCommand {
    let mut cmd = RodoCommand::None("less parameters".into());
    let args: Vec<String> = env::args().collect();

    if args.len() < 2_usize {
        return cmd;
    }

    let subcommand = args[1].as_str();
    match subcommand {
        "-h" | "--help"    => cmd = RodoCommand::Help,
        "-v" | "--version" => cmd = RodoCommand::Version,
        "add" => {
            if args.len() < 3_usize {
                cmd = RodoCommand::None("[add] subcommand need item".into());
            }
            else {
                cmd = RodoCommand::Add(args[2].clone());
            }
        }
        "list" => cmd = RodoCommand::List,
        "info" => cmd = RodoCommand::Info,
        "remove" => {
            if args.len() < 3_usize {
                cmd = RodoCommand::None("[remove] subcommand need sequence".into());
            }
            else {
                cmd = RodoCommand::Remove(args[2].clone());
            }
        }
        _ => cmd = RodoCommand::None(subcommand.to_string()),
    }

    return cmd;
}

fn handle_command (db: &mut TodoDB, cmd: &RodoCommand) -> Result<()> {
    match *cmd {
        RodoCommand::Add(ref msg)    => db.add_record(msg.as_str())?,
        RodoCommand::Remove(ref msg) => db.del_record(msg.as_str())?,
        RodoCommand::List => {
            for item in db.query() {
                LOGI!("{}", item);
            }
        },
        RodoCommand::Info => {
            LOGI!("Rodo version: {}", RODO_VERSION);
            LOGI!("Rodo is the simple Todo-List manager");
            LOGI!("Your Todo-List is stored at: {}", db.file_path);
        },
        RodoCommand::Version => {
            LOGI!("{}", RODO_VERSION);
        },
        RodoCommand::Help => show_help(),
        RodoCommand::None(ref msg) => return Err(Error::new(ErrorKind::Other, msg.to_string())),
    }

    Ok(())
}

/// main
fn main () {
    init_logger();

    let db = TodoDB::new(RODO_DB_PATH);
    if let Err(e) = db {
        LOGE!("construct TodoDB {}", e);
        return;
    }

    let cmd = parse_command();
    if let Err(e) = handle_command(&mut db.unwrap(), &cmd) {
        LOGE!("{}", e);
    }
}