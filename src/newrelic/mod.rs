use std::collections::BTreeMap;
use serde;
use serde_json;
use serde_json::value;
use serde_json::value::Value as SerdeValue;
use Stats;
use config::Config;

#[derive(Serialize, Deserialize, Debug)]
pub struct NewrelicBody {
    agent: NewrelicAgent,
    components: Vec<NewrelicComponent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewrelicAgent {
    host: String,
    pid: i64,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct NewrelicComponent {
    name: String,
    guid: String,
    duration: f64,
    metrics: BTreeMap<String, SerdeValue>,
}



//Use slices for this BTreeMap
pub fn get_metrics_map(stats: &Stats) -> BTreeMap<String, SerdeValue> {
    let mut stats_map = BTreeMap::new();

    stats_map.insert("Component/conn/Connections[Count]".to_string(), value::to_value(&stats.connections));

    return stats_map;
}


pub fn get_newrelic_body_json(stats: &Stats, config: &Config) -> String {

    let agent = build_agent(&config.db_host, 1234, "0.1".to_string());

    let component = NewrelicComponent {
        name: config.db_name.to_string(),
        guid: config.plugin_guid.to_string(),
        duration: config.polls_per_sec,
        metrics: get_metrics_map(&stats),
    };

    let body = NewrelicBody {
        agent: agent,
        components: vec![component]
    };

    return serde_json::to_string(&body).unwrap();
}

pub fn build_agent(host: &str, pid: i64, version: String) -> NewrelicAgent {
    NewrelicAgent {
        host: host.to_string(),
        pid: pid,
        version: version,
    }
}
