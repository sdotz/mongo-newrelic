#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

mod config;
mod newrelic;

#[macro_use(bson, doc)]
extern crate bson;

extern crate mongodb;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::CommandType;

use std::thread;
use std::time::Duration;
use std::io::Read;

#[macro_use] extern crate hyper;
use hyper::Client as HyperClient;
use hyper::header::Connection;
use hyper::header::{Headers, Accept, ContentType, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

header! { (XLicenseKey, "X-License-Key") => [String] }

#[derive(Debug, Copy, Clone)]
pub struct Stats {
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
    net_in_bytes: f64,
    net_out_bytes: f64,
    idx_miss_ratio: f64,
    r_time_locked_micros: i64,
    w_time_locked_micros: i64,
    docs_returned: i64,
    docs_inserted: i64,
}

fn main() {
    let config_path = "/Users/206637/Documents/mongo-newrelic/config.toml".to_owned();
    if let Ok(config) = config::get_config(&config_path) {
        let client = connect_db(&config.db_host, &config.db_user, &config.db_pwd);
        let db = client.db("admin");
        match db.auth_cr(&config.db_user, &config.db_pwd) {
            Ok(k) => println!("Success: {:?}", k),
            Err(e) => println!("Auth failed: {:?}", e),
        }

        let mut prev_stats: Option<Stats> = None;

        loop {
            if let Some(c_stats) = poll_stats(&db) {
                if let Some(p_stats) = prev_stats {
                    let computed_stats = diff_stats(p_stats, c_stats);
                    let body = newrelic::get_newrelic_body_json(&computed_stats, &config);
                    post_stats(body, &config.newrelic_api_url, &config.newrelic_license_key);
                    println!("computed: {:?}", computed_stats);
                    //println!("p_stats: {:?}", prev_stats);
                    //println!("stats: {:?}", c_stats);
                }
                prev_stats = Some(c_stats);
            }

            let seconds = config.poll_cadence_secs as u64;
            thread::sleep(Duration::new(seconds, 0));
        }
    };
}

fn connect_db(db_host: &str, db_user: &str, db_pwd: &str) -> Client {
    let client = Client::with_uri(db_host)
    .ok()
    .expect("Failed to initialize client.");

    client
}


//unwrap() unwraps an option yielding the content of a `Some`
fn poll_stats(db: &Database) -> Option<Stats> {

    let cmd = doc! { "serverStatus" => 1};
    let result = db.command(cmd, CommandType::Suppressed, None);

    if let Ok(r) = result {
        println!("{}", r);
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

        let locks = r.get_document("locks")
        .ok()
        .expect("Could not get locks node");

        let locks_for_db = locks.get_document(&db.name).unwrap().get_document("timeLockedMicros")
        .ok()
        .expect(
            &format!("Could not get timeLockedMicros for db {}", &db.name)
        );

        let doc_metrics = r.get_document("metrics").unwrap().get_document("document")
        .ok()
        .expect("Could not get metrics.document");

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
            net_in_bytes: network.get_f64("bytesIn").unwrap(),
            net_out_bytes: network.get_f64("bytesOut").unwrap(),
            idx_miss_ratio: index_counters.get_f64("missRatio").unwrap(),
            r_time_locked_micros: locks_for_db.get_i64("r").unwrap(),
            w_time_locked_micros: locks_for_db.get_i64("w").unwrap(),
            docs_returned: doc_metrics.get_i64("returned").unwrap(),
            docs_inserted: doc_metrics.get_i64("inserted").unwrap(),
        };

        Some(stats)
    } else {
        None
    }
}


fn diff_stats(p_stats: Stats, c_stats: Stats) -> Stats {
    Stats {
        connections: c_stats.connections,
        connections_available: c_stats.connections_available,
        active_r: c_stats.active_r,
        active_w: c_stats.active_w,
        inserts: c_stats.inserts - p_stats.inserts,
        queries: c_stats.queries - p_stats.queries,
        updates: c_stats.updates - p_stats.updates,
        deletes: c_stats.deletes - p_stats.deletes,
        getmores: c_stats.getmores - p_stats.getmores,
        commands: c_stats.commands - p_stats.commands,
        page_fault: c_stats.page_fault - p_stats.page_fault,
        queue_read: c_stats.queue_read,
        queue_write: c_stats.queue_write,
        net_in_bytes: c_stats.net_in_bytes - p_stats.net_in_bytes,
        net_out_bytes: c_stats.net_out_bytes - p_stats.net_out_bytes,
        idx_miss_ratio: c_stats.idx_miss_ratio,
        r_time_locked_micros: c_stats.r_time_locked_micros - p_stats.r_time_locked_micros,
        w_time_locked_micros: c_stats.w_time_locked_micros - p_stats.w_time_locked_micros,
        docs_returned: c_stats.docs_returned - p_stats.docs_returned,
        docs_inserted: c_stats.docs_inserted - p_stats.docs_inserted,
    }
}

fn post_stats(payload: String, newrelic_api_url: &str, newrelic_license_key: &str){

    println!("Payload: {}", payload);

    let client = HyperClient::new();

    let mut headers = Headers::new();
    headers.set(
        Accept(vec![
            qitem(Mime(TopLevel::Application, SubLevel::Json,
            vec![(Attr::Charset, Value::Utf8)])),
        ])
    );

    headers.set(
        ContentType(
            Mime(TopLevel::Application, SubLevel::Json, vec![(Attr::Charset, Value::Utf8)])
        )
    );

    headers.set(
        ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]))
    );

    headers.set(XLicenseKey(newrelic_license_key.to_owned()));

    let mut res = client.post(newrelic_api_url)
    .headers(headers)
    .body(&payload)
    .header(Connection::close())
    .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);
}


