use std::collections::BTreeMap;
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
    duration: i64,
    metrics: BTreeMap<String, SerdeValue>,
}

//Use slices for this BTreeMap
pub fn get_metrics_map(stats: &Stats) -> BTreeMap<String, SerdeValue> {
    let mut stats_map = BTreeMap::new();

    stats_map.insert("Component/conn/Connections[Count]".to_owned(), value::to_value(&stats.connections));
    stats_map.insert("Component/conn/Connections Available[Count]".to_owned(), value::to_value(&stats.connections_available));
    stats_map.insert("Component/clients/Active Readers[Count]".to_owned(), value::to_value(&stats.active_r));
    stats_map.insert("Component/clients/Active Writers[Count]".to_owned(), value::to_value(&stats.active_w));
    stats_map.insert("Component/ops/Inserts[Count]".to_owned(), value::to_value(&stats.inserts));
    stats_map.insert("Component/ops/Queries[Count]".to_owned(), value::to_value(&stats.queries));
    stats_map.insert("Component/ops/Updates[Count]".to_owned(), value::to_value(&stats.updates));
    stats_map.insert("Component/ops/Deletes[Count]".to_owned(), value::to_value(&stats.deletes));
    stats_map.insert("Component/ops/Getmores[Count]".to_owned(), value::to_value(&stats.getmores));
    stats_map.insert("Component/ops/Commands[Count]".to_owned(), value::to_value(&stats.commands));
    stats_map.insert("Component/sys/Page Fault[Count]".to_owned(), value::to_value(&stats.page_fault));
    stats_map.insert("Component/sys/Queue Read[Count]".to_owned(), value::to_value(&stats.queue_read));
    stats_map.insert("Component/sys/Queue Write[Count]".to_owned(), value::to_value(&stats.queue_write));
    stats_map.insert("Component/net/Bytes In[Count]".to_owned(), value::to_value(&stats.net_in_bytes));
    stats_map.insert("Component/net/Bytes Out[Count]".to_owned(), value::to_value(&stats.net_out_bytes));
    stats_map.insert("Component/idx/Index Miss Ratio[Count]".to_owned(), value::to_value(&stats.idx_miss_ratio));
    stats_map.insert("Component/locks/R Locked uSec[Count]".to_owned(), value::to_value(&stats.r_time_locked_micros));
    stats_map.insert("Component/locks/W Locked uSec[Count]".to_owned(), value::to_value(&stats.w_time_locked_micros));
    stats_map.insert("Component/locks/W Locked uSec[Count]".to_owned(), value::to_value(&stats.w_time_locked_micros));
    stats_map.insert("Component/documents/Returned[Count]".to_owned(), value::to_value(&stats.docs_returned));
    stats_map.insert("Component/documents/Inserted[Count]".to_owned(), value::to_value(&stats.docs_inserted));

    return stats_map;
}


pub fn get_newrelic_body_json(stats: &Stats, config: &Config) -> String {

    let agent = build_agent(&config.db_host, 1234, "0.1".to_owned());

    let component = NewrelicComponent {
        name: config.db_name.to_owned(),
        guid: config.plugin_guid.to_owned(),
        duration: config.poll_cadence_secs,
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
        host: host.to_owned(),
        pid: pid,
        version: version,
    }
}
