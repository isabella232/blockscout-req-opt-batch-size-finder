use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use reqwest::blocking::Client;

mod extention;

fn json_header() -> HeaderValue {
	HeaderValue::from_static("application/json; charset=utf-8")
}

/// eth_getBlockByNumber request
pub fn get_blocks_by_number(client:std::sync::Arc<Client>, node_end_point:&str, blocks:&[u64]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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
        .header(CONTENT_TYPE, json_header())
        .send()?;

    if res.status().is_client_error() || res.status().is_server_error()  {
        return Ok(vec!["0x0".into()]);
    }

    Ok(extention::parse_hashes_from_json(res.text()?, len))
}

/// eth_blockNumber request
pub fn get_block_number(client:std::sync::Arc<Client>, node_end_point:&str) -> Result<u64, Box<dyn std::error::Error>> {
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
        .header(CONTENT_TYPE, json_header())
        .send()?;
    
    let json: Resp = res.json()?;

    // println!("{:?}", res.text()?);
    // Ok(0)
    Ok(extention::from_hex_to_int(json.result.as_str()))
}

/// eth_getTransactionReceipt request
pub fn get_transactions_by_hash(client:std::sync::Arc<Client>, node_end_point:&str, transactions:&[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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
        .header(CONTENT_TYPE, json_header())
        .send()?;

    if res.status().is_client_error() || res.status().is_server_error()  {
        return Ok(vec!["0x0".into()]);
    }

    let ans = res.text()?;
    
    Ok(extention::parse_gas_from_json(ans, len))
}

