use std::env;

pub mod query;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // nodes for testing:
    // "https://core.poa.network/"
    // "https://rpc.xdaichain.com/"
    // "https://sokol.poa.network/"
    let node_end_point = "https://core.poa.network/";
    // total number of generated blocks
    let block_num_total = 40;
    // number of runs
    let cnt = 1;
    
    env_logger::init();

    query::start(node_end_point, block_num_total, cnt)
}

