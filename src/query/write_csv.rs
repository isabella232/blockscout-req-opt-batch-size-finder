extern crate csv;

use csv::Writer;

const HTTPS: &str = "https://";

pub fn create(node_name:String, stage:u64) -> Result<Writer<std::fs::File>, Box<dyn std::error::Error>> {
    let node_https: &str = &node_name[HTTPS.len()..node_name.len()];
    let node_https = node_https.replace('/', "");

    let name = match stage {
        0 => "_getBlocks",
        1 => "_getTransactions",
        _ => "_other",
    };

    let mut wtr = Writer::from_path(&format!("csv/{}{}.csv", &node_https, name))?;

    wtr.write_record(match stage {
        0 => &["block_batch_size", "batch_concurrency", "time"],
        _ => &["tx_batch", "tx_concurrency", "time"],
    })?;

    Ok(wtr)
}
