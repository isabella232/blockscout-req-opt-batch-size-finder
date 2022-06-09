use std::env;

pub mod query;

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Not enought arguments");
        return Ok(());
    }
    
    env_logger::init();
    
    // only with "https://"
    let node_end_point = args[1].to_string();

    // total number of generated blocks
    let block_num_total = args[2].parse::<usize>()?;

    // number of runs
    let cnt = match args.len() {
        4 => args[3].parse::<u64>()?,
        _ => 10,
    };

    query::start(node_end_point, block_num_total, cnt)
}

