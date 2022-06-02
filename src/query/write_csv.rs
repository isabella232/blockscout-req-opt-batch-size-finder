extern crate csv;

use csv::Writer;

const HTTPS: &'static str = "https://";

pub fn create(node_name:String, stage:u64) -> Result<Writer<std::fs::File>, Box<dyn std::error::Error>> {
    let node_https: &str = &node_name[HTTPS.len()..node_name.len() - 1];
    let mut wtr = Writer::from_path(&format!("csv/{}{}.csv", &node_https, stage))?;

    wtr.write_record(match stage {
        0 => &["block_batch_size", "batch_concurrency", "time"],
        _ => &["tx_batch", "tx_concurrency", "time"],
    })?;
    
    Ok(wtr)
}

// pub fn write_data(batch:usize, concurrency:usize, time:f64, writer:&Writer<std::fs::File>) -> Result<(), Box<dyn std::error::Error>> {
//     writer.write_record(&[format!("{batch}"), format!("{concurrency}"), format!("{time}")])?;

//     Ok(())
// }