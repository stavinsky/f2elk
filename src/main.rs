#[macro_use] extern crate serde_derive;

extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate reqwest;
extern crate ctrlc;

mod config;
use config::Config;
mod time;
mod file;
use file::file::get_files_by_pattern;

mod export;
use export::https::HttpsSender;
mod processing;
use processing::process_single_file;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn sig_handler() -> Arc<AtomicBool> {
    let term = Arc::new(AtomicBool::new(false));
    let r = term.clone();
    ctrlc::set_handler(move || {
        r.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    term
}

fn main_loop() -> Result<(), std::io::Error>{
    let term = sig_handler();
    let config_filename = "config.json".to_string();
    let config = match Config::from_file(&config_filename){
        Ok(cfg) => cfg,
        Err(e) => {
            println!(
                "can't read config file: {}\nError: {}",
                config_filename, e);
            ::std::process::exit(1);
        }
    };
    let files = get_files_by_pattern(
        &config.path, &config.file_pattern)?;
    let exporter = HttpsSender::new(
        &config.logstash_connection, config.logstash_ssl
    )?;
    for file in &files{
        process_single_file(&file, &config.db_file, exporter.to_owned(), &term);
    }
    Ok(())
}
fn main() -> Result<(), std::io::Error> {
    main_loop()?;
    Ok(())
}

