use reqwest::header::{HeaderValue, HeaderMap, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use reqwest::blocking::Client;

use log::{error};

use anyhow::anyhow;

mod extention;


fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("test optimal batch size"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"));
    headers
}

/// eth_blockNumber request
pub fn get_block_number(client: std::sync::Arc<Client>, node_end_point: String) -> Result<u64, anyhow::Error> {
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
        .send();

    if let Err(e) = res {
        return Err(error_handler(anyhow!(e)));
    }

    let json: Resp = res.unwrap().json()?;

    Ok(extention::from_hex_to_int(json.result.as_str()))
}

/// eth_getBlockByNumber request
pub fn get_blocks_by_number(client: std::sync::Arc<Client>, node_end_point: String, blocks: &[u64]) -> Result<Vec<String>, anyhow::Error> {
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
        .send();

    if let Err(e) = res {
        return Err(error_handler(anyhow!(e)));
    }

    let res = extention::parse_hashes_from_json(res.unwrap().text()?, len);

    if let Err(e) = res {
        return Err(error_handler(anyhow!(e)));
    }

    Ok(res.unwrap())
}

/// eth_getTransactionReceipt request
pub fn get_transactions_by_hash(client: std::sync::Arc<Client>, node_end_point: String, transactions: &[String]) -> Result<Vec<String>, anyhow::Error> {
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
        .send();

    if let Err(e) = res {
        return Err(error_handler(anyhow!(e)));
    }

    let ans = res.unwrap().text()?;
    
    let res = extention::parse_gas_from_json(ans, len);

    if let Err(e) = res {
        return Err(error_handler(anyhow!(e)));
    }

    Ok(res.unwrap())
}

fn error_handler(err: anyhow::Error) -> anyhow::Error {
    if let Some(e) = err.downcast_ref::<reqwest::Error>() {
        if e.is_redirect() {
            error!("server redirecting too many times or making loop");
            return err;
        } else if e.is_status() {
            let status = e.status().unwrap();
    
            if status.is_client_error() {
                error!("client error: {}", status);
            } else if status.is_server_error() {
                error!("server error: {}", status);
            } else if status.is_redirection() {
                error!("redirect: {}", status);
            }
            return err;
        } else if e.is_timeout() {
            error!("timeout: {}", e);
            return err;
        } else {
            error!("undefined error: {}", e);
            return err;
        }
    } else if let Some(e) = err.downcast_ref::<serde_json::Error>() {
        extention::error_handler(e);
        return err;
    } else {
        error!("can't downcast error: {}", err);
        return err;
    }
}