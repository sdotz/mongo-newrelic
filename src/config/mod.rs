extern crate toml;

use std::fs::File;
use std::io;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Config {
    pub db_host: String,
    pub db_name: String,
    pub poll_cadence_secs: i64,
    pub newrelic_api_url: String,
    pub newrelic_license_key: String,
    pub plugin_guid: String,
}

pub fn get_config(path: &str) -> Result<Config, io::Error> {
    let mut f = try!(File::open(&path));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));


    let config_toml: toml::Value = s.parse().unwrap();
    //println!("Result: {:?}", config_toml);

    Ok(
        Config {
            db_host:  config_toml.lookup("db_host").unwrap().as_str().unwrap().to_owned(),
            db_name:  config_toml.lookup("db_name").unwrap().as_str().unwrap().to_owned(),
            poll_cadence_secs:  config_toml.lookup("poll_cadence_secs").unwrap().as_integer().unwrap(),
            newrelic_api_url: config_toml.lookup("newrelic_api_url").unwrap().as_str().unwrap().to_owned(),
            newrelic_license_key: config_toml.lookup("newrelic_license_key").unwrap().as_str().unwrap().to_owned(),
            plugin_guid: config_toml.lookup("plugin_guid").unwrap().as_str().unwrap().to_owned(),
        }
    )
}

