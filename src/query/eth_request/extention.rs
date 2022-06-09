use serde_json::Value;

use log::error;

pub fn parse_hashes_from_json(json:String, batch_size:usize) -> Result<Vec<String>, serde_json::Error> {
    let objs: Value = serde_json::from_str(&json)?;
    let mut res: Vec<String> = vec![];

    for i in 0..batch_size {
         let r = objs[i]["result"]["transactions"].as_array().unwrap();

         for el in r {
             res.push(el.as_str().unwrap().into());
         }
    };

    Ok(res)
}

pub fn parse_gas_from_json(json:String, batch_size:usize) -> Result<Vec<String>, serde_json::Error> {
    let objs: Value = serde_json::from_str(&json)?;
    let mut res: Vec<String> = vec![];

    for i in 0..batch_size {
        res.push(objs[i]["result"]["gasUsed"].as_str().unwrap().into());
    };

    Ok(res)
}

pub fn from_hex_to_int(num:&str) -> u64 {
    let without_prefix = num.trim_start_matches("0x");
    u64::from_str_radix(without_prefix, 16).unwrap()
}

pub fn error_handler(err: &serde_json::Error) {
    if err.is_io() {
        error!("I/O stream error");
    } else if err.is_eof() {
        error!("EOF error");
    } else if err.is_syntax() {
        // with a large number of requests syntax error apears
        error!("syntax error: {}", err);
    } else if err.is_data() {
        error!("semantically incorrect dara: {}", err);
    } else {
        error!("undefined error: {}", err);
    }
}