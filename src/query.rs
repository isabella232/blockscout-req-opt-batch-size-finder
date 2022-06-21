mod eth_request;
mod time_data;

use std::sync::Arc;
use reqwest::blocking::Client;
use std::thread;

use std::time::{Instant};
use std::cmp;
use rand::Rng;
use log::{info, error};

/// Entry point
pub fn start(node_end_point:String, block_num_total:usize, cnt:u64) -> Result<(), reqwest::Error> {
    let node_end_point = get_good_url(&node_end_point);

    info!("Connecting to node {}", node_end_point);

    let client = Arc::new(Client::new());
    let max_block_number = eth_request::get_block_number(Arc::clone(&client), &node_end_point)?;
    let mut ans_hash = vec![];
    let mut is_first = true;

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

    info!("eth_getBlockByNubmer rquest in progress.");
    info!("block_batch_size;block_concurrency;time");

    let cumulative_time = Instant::now();

    let mut timer = time_data::TimeData {node_end_point: node_end_point.to_string(), ..Default::default()};

    if let Err(e) = timer.init_write(0) {
        error!("error while creating csv. check your directory: {}", e);
        return Ok(());
    }

    for block_batch_size in (1..block_num_total+1).rev() {
        let block_concurrency = match block_num_total % block_batch_size {
            0 => block_num_total / block_batch_size,
            _ => block_num_total / block_batch_size + 1
        };
        timer.init();

        for _ in 0..cnt {
            timer.start();
            let mut handles = vec![];

            for thread_number in 0..block_concurrency {
                let right = cmp::min((thread_number + 1) * block_batch_size, block_num_total);
                let left = thread_number * block_batch_size;
                let mut thread_blocks = vec![0; right - left];

                thread_blocks.clone_from_slice(&blocks[left..right]);

                let ref_client = Arc::clone(&client);
                let ref_node = node_end_point.to_string();

                let handle = thread::spawn(move || {
                        eth_request::get_blocks_by_number(ref_client, ref_node, &thread_blocks)
                    });

                handles.push(handle);
            }

            for handle in handles {
                let res = handle.join().unwrap();

                if let Err(e) = res {
                    error!("error while threading: {:?}", e);
                    continue;
                }

                let res = res.unwrap();

                if is_first {
                    ans_hash.push(res);
                }
            }
            is_first = false;
            timer.iteration();
        }
        let fin_avg = timer.end(block_batch_size, block_concurrency).unwrap();

        info!("{};{};{}", block_batch_size, block_concurrency, fin_avg);
    }

    info!("Get timing data for eth_getBlockByNumber requests:");
    time_data::get_timing_data(timer);

    let mut timer = time_data::TimeData {node_end_point: node_end_point.to_string(), ..Default::default()};

    if let Err(e) = timer.init_write(1) {
        error!("error while creating csv. check your directory: {}", e);
        return Ok(());
    }

    info!("eth_getTransactionReceipt rquest in progress.");
    info!("tx_batch;tx_concurrency;time");

    let tx_hashes = &ans_hash[0];
    let num_of_hashes = tx_hashes.len();

    // To do
    // with smaller bound for tx_batch
    // every node throw *429 Too Many Requests*
    let tx_batch_min = 1;

    for tx_batch in (tx_batch_min..num_of_hashes+1).rev() {
        let tx_concurrency = match num_of_hashes % tx_batch {
            0 => num_of_hashes / tx_batch,
            _ => num_of_hashes / tx_batch + 1
        };

        timer.init();

        for _ in 0..cnt {
            timer.start();
            let mut handles = vec![];

            for thread_number in 0..tx_concurrency {
                let right = cmp::min((thread_number + 1) * tx_batch, num_of_hashes);
                let left = thread_number * tx_batch;
                let mut thread_hashes = vec!["".to_string(); right - left];

                thread_hashes.clone_from_slice(&tx_hashes[left..right]);

                let ref_client = Arc::clone(&client);
                let ref_node = node_end_point.to_string();

                let handle = thread::spawn(move || {
                        eth_request::get_transactions_by_hash(Arc::clone(&ref_client), ref_node, &thread_hashes)
                    });

                handles.push(handle);
            }

            for handle in handles {
                let res = handle.join().unwrap();

                if let Err(e) = res {
                    error!("error while threading: {:?}", e);
                    continue;
                }
            }

            timer.iteration();
        }
        let fin_avg = timer.end(tx_batch, tx_concurrency).unwrap();

        info!("{};{};{}", tx_batch, tx_concurrency, fin_avg);
    }

    info!("Get timing data for eth_getTransactionReceipt requests:");
    time_data::get_timing_data(timer);

    info!("Cumulative time for: block_num_total={} num_of_hashes={} number_of_runs={}",
          block_num_total, num_of_hashes, cnt);
    info!("{} seconds",  cumulative_time.elapsed().as_secs());
    Ok(())
}

fn get_good_url(node: &String) -> String {
    if node.starts_with(eth_request::HTTPS) {
        node.to_string()
    } else {
        format!("{}{node}", eth_request::HTTPS)
    }
}
