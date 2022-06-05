mod eth_request;
mod timing;
mod write_csv;

use std::sync::Arc;
use reqwest::blocking::Client;
use std::thread;

use std::time::{Instant};
use std::cmp;
use rand::Rng;
use log::info;

/// entry point
pub fn start(node_end_point:String, block_num_total:usize, cnt:u64) -> Result<(), Box<dyn std::error::Error>> {
    info!("Connecting to node {}", node_end_point);

    let client = Arc::new(Client::new());

    let ref_node = node_end_point.to_string();

    let max_block_number = eth_request::get_block_number(Arc::clone(&client), ref_node)?;

    info!("Connection succeed");
    info!("max_block_number: {}", max_block_number);

    // default value (optional)
    let block_range = max_block_number;
    info!("block_range: {}", block_range);

    // generating random block numbers
    let mut rng = rand::thread_rng();
    let blocks: Vec<u64> = (0..block_num_total).map(|_| rng.gen_range(0..block_range)).collect();

    info!("block_num_total: {}", block_num_total);
    info!("List of generated random blocks:\n{:?}", blocks);

    info!("Number of runs: {}", cnt);

    let mut blocks_timing = timing::Timing {data: Vec::new()};
    let mut hashes_timing = timing::Timing {data: Vec::new()};

    let mut ans_hash = vec![];
    let mut is_first = true;

    info!("eth_getBlockByNubmer rquest in progress.");
    info!("block_batch_size;block_concurrency;time");

    let mut writer = write_csv::create(node_end_point.to_string(), 0)?;

    let cumulative_time = Instant::now();

    for block_batch_size in (1..block_num_total+1).rev() {
        let block_concurrency = match block_num_total % block_batch_size {
            0 => block_num_total / block_batch_size,
            _ => block_num_total / block_batch_size + 1
        };
        let mut avg = 0;

        for _ in 0..cnt {
            let now = Instant::now();
            let mut handles = vec![];

            for thread_number in 0..block_concurrency {
                let right = cmp::min((thread_number + 1) * block_batch_size, block_num_total);
                let left = thread_number * block_batch_size;
                let mut thread_blocks = vec![0; right - left];

                thread_blocks.clone_from_slice(&blocks[left..right]);

                let ref_client = Arc::clone(&client);

                let ref_node = node_end_point.to_string();

                let handle = thread::spawn(move || {
                        eth_request::get_blocks_by_number(ref_client, ref_node, &thread_blocks).unwrap_or_else(|_| vec!["0".into()])
                    });

                handles.push(handle);
            }

            for handle in handles {
                let res = handle.join().unwrap_or_else(|_| vec!["0".into()]);

                if is_first {
                    ans_hash.push(res);
                }
            }
            is_first = false;
            avg += now.elapsed().as_millis();
        }
        let fin_avg = (avg as f64) / (cnt as f64);
        info!("{};{};{}", block_batch_size, block_concurrency, fin_avg);
        blocks_timing.data.push(fin_avg);

        writer.write_record(&[format!("{block_batch_size}"), format!("{block_concurrency}"), format!("{fin_avg}")])?;
    }
    info!("Get timing data for eth_getBlockByNumber requests:");

    let block_batch_indexes = timing::get_timing_data(&blocks_timing);

    let block_batch_min = block_num_total - block_batch_indexes.0 + 1;
    let remainder_0 = if block_num_total % block_batch_min == 0 {0} else {1};
    let block_concurrency_min = block_num_total / (block_num_total - block_batch_indexes.0 + 1) + remainder_0;
    info!("Minimum with block_batch_size={} and block_concurrency={}", block_batch_min, block_concurrency_min);

    let block_batch_max = block_num_total - block_batch_indexes.1 + 1;
    let remainder_1 = if block_num_total % block_batch_max == 0 {0} else {1};
    let block_concurrency_max = block_num_total / (block_num_total - block_batch_indexes.1 + 1) + remainder_1;
    info!("Maximum with block_batch_size={} and block_concurrency={}", block_batch_max, block_concurrency_max);

    let time_min = &blocks_timing.data[block_num_total - block_batch_min];
    let time_max = &blocks_timing.data[block_num_total - block_batch_max];

    writer.write_record(&[format!("{block_batch_max}"), format!("{block_concurrency_max}"), format!("{time_max}")])?;
    writer.write_record(&[format!("{block_batch_min}"), format!("{block_concurrency_min}"), format!("{time_min}")])?;
    writer.flush()?;

    info!("eth_getTransactionReceipt rquest in progress.");
    info!("tx_batch;tx_concurrency;time");

    writer = write_csv::create(node_end_point.to_string(), 1)?;

    let tx_hashes = &ans_hash[0];
    let num_of_hashes = tx_hashes.len();

    for tx_batch in (1..num_of_hashes+1).rev() {
        let tx_concurrency = match num_of_hashes % tx_batch {
            0 => num_of_hashes / tx_batch,
            _ => num_of_hashes / tx_batch + 1
        };

        let mut avg = 0;
        for _ in 0..cnt {
            let now = Instant::now();

            let mut handles = vec![];

            for thread_number in 0..tx_concurrency {
                let right = cmp::min((thread_number + 1) * tx_batch, num_of_hashes);
                let left = thread_number * tx_batch;
                let mut thread_hashes = vec!["".to_string(); right - left];

                thread_hashes.clone_from_slice(&tx_hashes[left..right]);

                let ref_client = Arc::clone(&client);

                let ref_node = node_end_point.to_string();

                let handle = thread::spawn(move || {
                        eth_request::get_transactions_by_hash(Arc::clone(&ref_client), ref_node, &thread_hashes).unwrap_or_else(|_| vec!["0".into()])
                    });

                handles.push(handle);
            }

            for handle in handles {
                let _res = handle.join().unwrap_or_else(|_| vec!["0".into()]);
            }

            avg += now.elapsed().as_millis();
        }
        let fin_avg = (avg as f64) / (cnt as f64);

        info!("{};{};{}", tx_batch, tx_concurrency, fin_avg);
        hashes_timing.data.push(fin_avg);

        writer.write_record(&[format!("{tx_batch}"), format!("{tx_concurrency}"), format!("{fin_avg}")])?;
    }

    info!("Get timing data for eth_getTransactionReceipt requests:");

    let tx_batch_indexes = timing::get_timing_data(&hashes_timing);

    let tx_batch_min = num_of_hashes - tx_batch_indexes.0 + 1;
    let remainder_0 = if num_of_hashes % tx_batch_min == 0 {0} else {1};
    let tx_concurrency_min = num_of_hashes / tx_batch_min + remainder_0;
    info!("Minimum with tx_batch={} and tx_concurrency={}", tx_batch_min, tx_concurrency_min);

    let tx_batch_max = num_of_hashes - tx_batch_indexes.1 + 1;
    let remainder_1 = if num_of_hashes % tx_batch_max == 0 {0} else {1};
    let tx_concurrency_max = num_of_hashes / tx_batch_max + remainder_1;
    info!("Maximum with tx_batch={} and tx_concurrency={}", tx_batch_max, tx_concurrency_max);

    let time_min = &hashes_timing.data[num_of_hashes - tx_batch_min];
    let time_max = &hashes_timing.data[num_of_hashes - tx_batch_max];

    writer.write_record(&[format!("{tx_batch_max}"), format!("{tx_concurrency_max}"), format!("{time_max}")])?;
    writer.write_record(&[format!("{tx_batch_min}"), format!("{tx_concurrency_min}"), format!("{time_min}")])?;
    writer.flush()?;

    info!("Cumulative time for:\nblock_num_total={}\nnum_of_hashes={}\nnumber_of_runs={}\n{} seconds",
          block_num_total, num_of_hashes, cnt, cumulative_time.elapsed().as_secs());
    Ok(())
}
