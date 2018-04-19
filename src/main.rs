#![feature(type_ascription)]

mod filesystem;
use filesystem::Filesystem;

// extern crate fuse;
extern crate fuse_mt;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate clap;
extern crate rusqlite;
extern crate time;
extern crate libc;

use simplelog::{Config, LogLevelFilter, TermLogger};

use clap::{Arg, App};

use std::env::home_dir;
use std::path::PathBuf;

fn main() {
    let matches = App::new("PlayFS")
        .about("A fast Fuse Filesystem for Beets")
        .version("0.1")
        .author("Rémi NICOLE <minijackson@riseup.net>")
        .arg(Arg::with_name("mountpoint")
             .help("Where to mount the filesystem")
             .required(true)
             .index(1))
        .arg(Arg::with_name("database")
             .help("The location of the Beets database file. [default: ~/.config/beets/library.db]")
             .value_name("file")
             .long("db")
             .short("d"))
        .arg(Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("Sets the level of verbosity"))
        .get_matches();

    TermLogger::init(match matches.occurrences_of("v") {
                         0 => LogLevelFilter::Warn,
                         1 => LogLevelFilter::Info,
                         2 => LogLevelFilter::Debug,
                         3 | _ => LogLevelFilter::Trace,
                     },
                     Config::default())
            .unwrap();

    let mountpoint = matches.value_of("mountpoint").unwrap();
    let db = matches
        .value_of("database")
        .map(|path| PathBuf::from(path))
        .unwrap_or_else(|| {
                            let home = home_dir()
                                .expect("Impossible to retrieve the home directory");
                            home.join(".config/beets/library.db")
                        });

    let fs = Filesystem::new(&db);

    fuse_mt::mount(fuse_mt::FuseMT::new(fs, 1), &mountpoint, &[]).unwrap();
}
