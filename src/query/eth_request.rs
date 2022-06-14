use reqwest::header::{HeaderValue, HeaderMap, CONTENT_TYPE, USER_AGENT};
use reqwest::blocking::Client;

use log::{error};

mod extentions;


fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("test optimal batch size"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"));
    headers
}

/// eth_blockNumber request
pub fn get_block_number(client: std::sync::Arc<Client>, node_end_point: String) -> Result<u64, reqwest::Error> {
    let arg = r#"{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":"83"}"#;

    let res = client.post(&node_end_point)
        .body(arg)
        .headers(construct_headers())
        .send();

    if let Err(e) = res {
        return Err(error_handler(e));
    }

    let json: Result<extentions::Response, reqwest::Error> = res.unwrap().json();

    if let Err(e) = json {
        Err(error_handler(e))
    } else {
        let result = json.unwrap().result;

        if let extentions::RequestObj::Error(err) = result {
            error!("error from EVM with code {}: {}", err.code, err.message);
            Ok(0)
        } else if let extentions::RequestObj::MaxBlock(ans) = result {
            Ok(extentions::from_hex_to_int(&ans))
        } else {
            error!("undefined behaviour");
            Ok(0)
        }
    }
}

/// eth_getBlockByNumber request
pub fn get_blocks_by_number(client: std::sync::Arc<Client>, node_end_point: String, blocks: &[u64]) -> Result<Vec<String>, reqwest::Error> {
    let mut arg: String = "[".into();

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
        return Err(error_handler(e));
    }

    let jsons: Result<Vec<extentions::Response>, reqwest::Error> = res.unwrap().json();
    // let jsons: Result<std::string::String, reqwest::Error> = res.unwrap().text();

    // println!("{:?}", jsons);

    if let Err(e) = jsons {
        return Err(error_handler(e));
    }
    
    // Ok(vec![])

    Ok(jsons.unwrap()
            .into_iter()
            .flat_map(extentions::get_hashes)
            .collect())
}

/// eth_getTransactionReceipt request
pub fn get_transactions_by_hash(client: std::sync::Arc<Client>, node_end_point: String, transactions: &[String]) -> Result<Vec<String>, reqwest::Error> {
    let mut arg: String = "[".into();

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
        return Err(error_handler(e));
    }

    let jsons: Result<Vec<extentions::Response>, reqwest::Error> = res.unwrap().json();
    // let jsons: Result<std::string::String, reqwest::Error> = res.unwrap().text();

    // println!("{:?}", jsons);

    if let Err(e) = jsons {
        return Err(error_handler(e));
    }


    Ok(jsons.unwrap()
            .into_iter()
            .map(extentions::get_gas)
            .collect())

    // Ok(vec![])
}

fn error_handler(e: reqwest::Error) -> reqwest::Error {
    if e.is_redirect() {
        error!("server redirecting too many times or making loop");
        e
    } else if e.is_status() {
        let status = e.status().unwrap();

        if status.is_client_error() {
            error!("client error: {}", status);
        } else if status.is_server_error() {
            error!("server error: {}", status);
        } else if status.is_redirection() {
            error!("redirect: {}", status);
        }
        e
    } else if e.is_timeout() {
        error!("timeout: {}", e);
        e
    } else if e.is_decode() {
        error!("problem decode information {}", e);
        e
    } else {
        error!("undefined error: {}", e);
        e
    }
}