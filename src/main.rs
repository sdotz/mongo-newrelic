mod config;

#[macro_use(bson, doc)]
extern crate bson;
use bson::Bson;

extern crate mongodb;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::CommandType;

use std::thread;
use std::time::Duration;
use std::io::Read;

extern crate hyper;
use hyper::Client as HyperClient;
use hyper::header::Connection;

extern crate serde;
extern crate serde_json;

#[derive(Debug)]
struct Stats {
    connections: i32,
    connections_available: i32,
    active_r: i32,
    active_w: i32,
    inserts: i32,
    queries: i32,
    updates: i32,
    deletes: i32,
    getmores: i32,
    commands: i32,
    page_fault: i32,
    queue_read: i32,
    queue_write: i32,
    net_in_bytes: i32,
    net_out_bytes: i32,
    idx_miss_ratio: f64,
}

fn main() {
    let config_path = "/Users/206637/Documents/mongo-newrelic/config.toml".to_string();
    if let Ok(config) = config::get_config(&config_path) {
        let client = connect_db(&config.db_host);
        let db = client.db(&config.db_name);

        loop {
            if let Some(stats) = poll_stats(&db) {
                post_stats(stats, &config);
            }
            thread::sleep(Duration::new(1, 0));
        }
    };

}

fn connect_db(db_host: &str) -> Client {
    let client = Client::connect(db_host, 27017)
    .ok()
    .expect("Failed to initialize client.");

    client
}


//unwrap() unwraps an option yielding the content of a `Some`
fn poll_stats(db: &Database) -> Option<Stats> {
    println!("Poll!");
    let cmd = doc! { "serverStatus" => 1 };

    let result = db.command(cmd, CommandType::Suppressed, None);

    if let Ok(r) = result {
        let connections = r.get_document("connections")
            .ok()
            .expect("Could not get connections node");

        let global_lock = r.get_document("globalLock")
            .ok()
            .expect("Could not get globalLock node");

        let opcounters = r.get_document("opcounters")
            .ok()
            .expect("Could not get opcounters node");

        let network = r.get_document("network")
            .ok()
            .expect("Could not get network node");

        let index_counters = r.get_document("indexCounters")
            .ok()
            .expect("Could not get network node");

        let record_stats = r.get_document("recordStats")
            .ok()
            .expect("Could not get recordStats node");

        let stats = Stats {
            connections: connections.get_i32("current").unwrap(),
            connections_available: connections.get_i32("available").unwrap(),
            active_r: global_lock.get_document("activeClients").unwrap().get_i32("readers").unwrap(),
            active_w: global_lock.get_document("activeClients").unwrap().get_i32("writers").unwrap(),
            inserts:  opcounters.get_i32("insert").unwrap(),
            queries: opcounters.get_i32("query").unwrap(),
            updates: opcounters.get_i32("update").unwrap(),
            deletes: opcounters.get_i32("delete").unwrap(),
            getmores: opcounters.get_i32("getmore").unwrap(),
            commands: opcounters.get_i32("command").unwrap(),
            page_fault: record_stats.get_i32("pageFaultExceptionsThrown").unwrap(),
            queue_read: global_lock.get_document("currentQueue").unwrap().get_i32("readers").unwrap(),
            queue_write: global_lock.get_document("currentQueue").unwrap().get_i32("writers").unwrap(),
            net_in_bytes: network.get_i32("bytesIn").unwrap(),
            net_out_bytes: network.get_i32("bytesOut").unwrap(),
            idx_miss_ratio: index_counters.get_f64("missRatio").unwrap()
        };

        Some(stats)
    } else {
        None
    }
}


fn diff_stats(before: Stats, after: Stats) {

}

fn post_stats(stats: Stats, config: &config::Config){
    println!("stats: {:?}", stats);

    let mut client = HyperClient::new();

    let mut res = client.post(&config.newrelic_api_url)
    .body("foo=bar")
    .header(Connection::close())
    .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);
}




