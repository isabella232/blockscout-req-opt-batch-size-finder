use reqwest::header::{HeaderValue, HeaderMap, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use reqwest::blocking::Client;

use log::{error};

mod extention;

// fn json_header() -> HeaderValue {
// 	HeaderValue::from_static("application/json; charset=utf-8")
// }

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("test"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"));
    headers
}

/// eth_blockNumber request
pub fn get_block_number(client:std::sync::Arc<Client>, node_end_point:String) -> Result<u64, Box<dyn std::error::Error>> {
    #[allow(dead_code)]
    #[derive(Deserialize)]
    struct Resp {
        jsonrpc: String,
        result: String,
        id: String,
    }
    
    let arg = r#"{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":"83"}"#;

    let res = client.post(&node_end_point)
        .body(arg)
        .headers(construct_headers())
        .send()?;
    
    let json: Resp = res.json()?;

    Ok(extention::from_hex_to_int(json.result.as_str()))
}

/// eth_getBlockByNumber request
pub fn get_blocks_by_number(client:std::sync::Arc<Client>, node_end_point:String, blocks:&[u64]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut arg: String = "[".into();
    let len = blocks.len();

    for block in blocks {
        let args: &str = &format!(r#"["0x{:x}", false]"#, block);

        arg += &format!(r#"{{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":{},"id":"1"}},"#, args); 
    }

    arg.pop();
    arg += "]";

    let res = client.post(&node_end_point)
        .body(arg)
        .headers(construct_headers())
        .send()?;

    if res.status().is_client_error() || res.status().is_server_error()  {
        error!("Error while eth_getBlockByNumber");
        error!("{:?}", res.text()?);
        return Ok(vec!["0x0".into()]);
    }

    Ok(extention::parse_hashes_from_json(res.text()?, len))
}

/// eth_getTransactionReceipt request
pub fn get_transactions_by_hash(client:std::sync::Arc<Client>, node_end_point:String, transactions:&[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut arg: String = "[".into();
    let len = transactions.len();

    for transaction in transactions {
        let args: &str = &format!(r#"["{}"]"#, transaction);

        arg += &format!(r#"{{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":{},"id":"1"}},"#, args); 
    }

    arg.pop();
    arg += "]";

    let res = client.post(&node_end_point)
        .body(arg)
        .headers(construct_headers())
        .send()?;

    if res.status().is_client_error() || res.status().is_server_error()  {
        error!("Error while eth_getTransactionReceipt");
        error!("{:?}", res.text()?);
        return Ok(vec!["0x0".into()]);
    }

    let ans = res.text()?;
    
    Ok(extention::parse_gas_from_json(ans, len))
}

