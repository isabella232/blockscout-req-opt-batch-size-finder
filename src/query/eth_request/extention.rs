use serde_json::Value;

pub fn parse_hashes_from_json(json:String, batch_size:usize) -> Vec<String> {
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

pub fn parse_gas_from_json(json:String, batch_size:usize) -> Vec<String> {
    let objs: Value = serde_json::from_str(&json).unwrap();
    let mut res: Vec<String> = vec![];

    for i in 0..batch_size {
        res.push(objs[i]["result"]["gasUsed"].as_str().unwrap().into());
    };

    res
}

pub fn from_hex_to_int(num:&str) -> u64 {
    let without_prefix = num.trim_start_matches("0x");
    u64::from_str_radix(without_prefix, 16).unwrap()
}
