use std::thread;
use std::sync::Arc;

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

use std::time::{Instant};
use std::cmp;
use rand::Rng;

struct Timing {
    /// timing in seconds
    data: Vec<f64>
}

fn get_timing_data(obj: &Timing) -> (usize, usize){
    let mut min = 100000.;
    let mut max = 0.;
    let mut min_index = 0;
    let mut max_index = 0;

    for (j, &value) in obj.data.iter().enumerate() {
        if value > max {
            max = value;
            max_index = j + 1;
        }
        
        if value < min {
            min = value;
            min_index = j + 1;
        }
    }

    println!("min: {}; max: {}; avg: {}", min, max, obj.data.iter().sum::<f64>() as f64 / obj.data.len() as f64);
    return (min_index, max_index)
}

fn parse_hashes_from_json(json:String, batch_size:usize) -> Vec<String> {
    let objs: Value = serde_json::from_str(&json).unwrap();
    let mut res: Vec<String> = vec![];

    for i in 0..batch_size {
         let r = objs[i]["result"]["transactions"].as_array().unwrap();

         for el in r {
             res.push(el.as_str().unwrap().into());
         }
    };

    res
}

fn parse_gas_from_json(json:String, batch_size:usize) -> Vec<String> {
    let objs: Value = serde_json::from_str(&json).unwrap();
    let mut res: Vec<String> = vec![];

    for i in 0..batch_size {
        res.push(objs[i]["result"]["gasUsed"].as_str().unwrap().into());
    };

    res
}

/// eth_getBlockByNumber request
fn get_blocks_by_number(client:std::sync::Arc<Client>, node_end_point:&str, blocks:&[u64]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut arg: String = "[".into();
    let len = blocks.len();

    for block in blocks {
        let args: &str = &format!(r#"["0x{:x}", false]"#, block);

        arg += &format!(r#"{{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":{},"id":"1"}},"#, args); 
    }

    arg.pop();
    arg += "]";

    let res = client.post(node_end_point)
        .body(arg)
        .send()?;

    if res.status().is_client_error() || res.status().is_server_error()  {
        return Ok(vec!["0x0".into()]);
    }

    Ok(parse_hashes_from_json(res.text()?, len))
}

/// eth_blockNumber request
fn get_block_number(client:std::sync::Arc<Client>, node_end_point:&str) -> Result<u64, Box<dyn std::error::Error>> {
    #[allow(dead_code)]
    #[derive(Deserialize)]
    struct Resp {
        jsonrpc: String,
        result: String,
        id: String,
    }
    
    let arg = r#"{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":"83"}"#;

    let res = client.post(node_end_point)
        .body(arg)
        .send()?;
    
    
    let json: Resp = res.json()?;

    Ok(from_hex_to_int(json.result.as_str()))
}

/// eth_getTransactionReceipt request
fn get_transactions_by_hash(client:std::sync::Arc<Client>, node_end_point:&str, transactions:&[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut arg: String = "[".into();
    let len = transactions.len();

    for transaction in transactions {
        let args: &str = &format!(r#"["{}"]"#, transaction);

        arg += &format!(r#"{{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":{},"id":"1"}},"#, args); 
    }

    arg.pop();
    arg += "]";

    let res = client.post(node_end_point)
        .body(arg)
        .send()?;

    if res.status().is_client_error() || res.status().is_server_error()  {
        return Ok(vec!["0x0".into()]);
    }

    let ans = res.text()?;
    
    Ok(parse_gas_from_json(ans, len))
}

fn from_hex_to_int(num:&str) -> u64 {
    let without_prefix = num.trim_start_matches("0x");
    u64::from_str_radix(without_prefix, 16).unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // nodes for testing:
    // "https://core.poa.network/"
    // "https://rpc.xdaichain.com/"
    // "https://sokol.poa.network/"
    let node_end_point = "https://rpc.xdaichain.com/";
    // total number of generated blocks
    let block_num_total = 40;
    // number of runs
    let cnt = 50;

    println!("Connecting to node {}", node_end_point);

    let client = Arc::new(Client::new());

    let max_block_number = get_block_number(Arc::clone(&client), node_end_point)?;

    println!("Connection succeed");
    println!("max_block_number: {}", max_block_number);

    // default value (optional)
    let block_range = max_block_number;
    println!("block_range: {}", block_range);

    // generating random block numbers
    let mut rng = rand::thread_rng();
    let blocks: Vec<u64> = (0..block_num_total).map(|_| rng.gen_range(0..block_range)).collect();

    println!("block_num_total: {}", block_num_total);
    println!("List of generated random blocks:\n{:?}", blocks);
    
    println!("Number of runs: {}", cnt);

    let mut ans_hash = vec![];
    let num_of_hashes;
    let mut is_first = true;

    let mut blocks_timing = Timing {data: Vec::new()};
    let mut hashes_timing = Timing {data: Vec::new()};

    println!("eth_getBlockByNubmer rquest in progress.");
    println!("block_batch_size;block_concurrency;time");

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
    
                let handle = thread::spawn(move || {
                        get_blocks_by_number(ref_client, node_end_point, &thread_blocks).unwrap_or(vec!["0".into()])
                    });
    
                handles.push(handle);
            }
    
            for handle in handles {
                let res = handle.join().unwrap_or(vec!["0".into()]);
                
                if is_first {
                    ans_hash.push(res);
                }
            }
            is_first = false;    
            avg += now.elapsed().as_millis();
        }
        let fin_avg = (avg as f64) / (cnt as f64);
        println!("{};{};{}", block_batch_size, block_concurrency, fin_avg);
        blocks_timing.data.push(fin_avg);
    }

    println!("eth_getTransactionReceipt rquest in progress.");
    println!("tx_batch;tx_concurrency;time");

    let tx_hashes = &ans_hash[0];
    num_of_hashes = tx_hashes.len();

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
    
                let handle = thread::spawn(move || {
                        get_transactions_by_hash(Arc::clone(&ref_client), node_end_point, &thread_hashes).unwrap_or(vec!["0".into()])
                    });
    
                handles.push(handle);
            }
    
            for handle in handles {
                let _res = handle.join().unwrap_or(vec!["0".into()]);
            }   

            avg += now.elapsed().as_millis();
        }
        let fin_avg = (avg as f64) / (cnt as f64);
        
        println!("{};{};{}", tx_batch, tx_concurrency, fin_avg);
        hashes_timing.data.push(fin_avg);
    }

    println!("Cumulative time for block_num_total={}, num_of_hashes={}, number_of_runs={} is: {} seconds\n", block_num_total, num_of_hashes, cnt,
                                                                                                     cumulative_time.elapsed().as_secs());
    println!("Get timing data for eth_getBlockByNumber requests:");
    let block_batch_indexes = get_timing_data(&blocks_timing);

    let remainder_0 = if block_num_total % (block_num_total - block_batch_indexes.0 + 1) == 0 {0} else {1};

    let remainder_1 = if block_num_total % (block_num_total - block_batch_indexes.1 + 1) == 0 {0} else {1};

    println!("Minimum with block_batch_size={} and block_concurrency={}", block_num_total - block_batch_indexes.0 + 1,
                                                                            block_num_total / (block_num_total - block_batch_indexes.0 + 1) + remainder_0);
    println!("Maximum with block_batch_size={} and block_concurrency={}", block_num_total - block_batch_indexes.1 + 1,
                                                                            block_num_total / (block_num_total - block_batch_indexes.1 + 1) + remainder_1);

    println!("");
    println!("Get timing data for eth_getTransactionReceipt requests:");
    let tx_batch_indexes = get_timing_data(&hashes_timing);    

    let remainder_0 = if num_of_hashes % (num_of_hashes - tx_batch_indexes.0 + 1) == 0 {0} else {1};

    let remainder_1 = if num_of_hashes % (num_of_hashes - tx_batch_indexes.1 + 1) == 0 {0} else {1};

    println!("Minimum with tx_batch={} and tx_concurrency={}", num_of_hashes - tx_batch_indexes.0 + 1,
                                                                num_of_hashes / (num_of_hashes - tx_batch_indexes.0 + 1) + remainder_0);
    println!("Maximum with tx_batch={} and tx_concurrency={}", num_of_hashes - tx_batch_indexes.1 + 1,
                                                                num_of_hashes / (num_of_hashes - tx_batch_indexes.1 + 1) + remainder_1);   

    Ok(())
}

