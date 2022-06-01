# Script to define optimal batch size and concurrency of blocks import

## Run
Pass the arguments in command line:
```
RUST_LOG=error cargo run node_end_point block_num_total cnt
```
Where:  
- `RUST_LOG=error` is optional (for tracking errors in requests)  
- `node_end_point` is node for test (e. g. *https://rpc.xdaichain.com/*)  
- `block_num_total` is number of generated blocks  
- `cnt` is optional (10 by default)  

## Tools
The work used: *rust* (rustc, cargo 1.60.0), python3. Part of the *Cargo.toml*:
```
[dependencies]
rand = "0.8.4"
reqwest = { version = "0.11", features = ["json", "blocking"] }
serde = {version = "1.0.137", features = ["derive"] }
serde_json = "1.0.81"
env_logger = "0.9.0"
log = "0.4.17"
```
## Structure of concurrency 
The picture shows how the block numbers are stored in memory and how concurrency is applied to them:

![](https://i.imgur.com/qCOH6eB.png)

## Analysis
*Images will be soon*  
Let's take a look at the distribution when iterating `block_batch_size`:


* [This](https://drive.google.com/file/d/1SQyoQ2U6RJpGpLjJlGq5rJplcrb8S5f-/view?usp=sharing) and [this](https://drive.google.com/file/d/1nhLULLho_H7XZpWFDIRBe6-KxDlD_rqa/view?usp=sharing) plots shows dependence of time on concurrency. 
Vertical line is the num of cores (I have 8).
As we can see, when script create more than 8 green threads, the scheduler makes a big contribution to performance. It also applies to *eth_getTransactionReceipt* request ([plot](https://drive.google.com/file/d/1sxhphiO1PMvkc8ENhT0iizDO798ixc7r/view?usp=sharing))
* [Here](https://drive.google.com/file/d/1F9Y6dvro36ni9CbtgVeUfUXLxp_HXi9u/view?usp=sharing) and [here](https://drive.google.com/file/d/1WT5pGkpKxdPeeF0MkoTTfAmseNqtQJqe/view?usp=sharing) we can see graphs for https://sokol.poa.network/ node.
Analyzing them, we can put forward a hypothesis about the best enumeration of variables.
One of the hypothesis is: *change varible `block_concurrency`, thus, go by divisors of `block_num_total`*.
* [Here](https://drive.google.com/file/d/1JznLpjqghJHBSovnREypRWaP9HNy_kyg/view?usp=sharing) graph for https://rpc.xdaichain.com/ node. For *eth_getBlockByNumber requests* we can see two other minimums, not only (10, 4). There are (7, 6) and (15, 3).

## Problems
* With a large number of requests to the node, sometimes the server gives an error [429 Too Many Requests](https://developer.mozilla.org/ru/docs/Web/HTTP/Status/429). In this case, the script works fine, skipping these requests.
* When the script is running for a long time (with `cnt`>=40) sometimes an error is issued (*TimedOut*). Now I'm trying to catch this error.
## Results
*Big table with images for different `node_end_point` will be here soon...*

## Сonclusion
Input variables are set in the script itself, but it can be easily fixed.
Among them: `node_end_point`, `block_num_total`, `cnt` (number of runs), `block_range`.
Two different versions of the script were written, their difference is in the approach to number of runs. In one of them hole script with *eth_getBlockByNumber* and *eth_getTransactionReceipt* request repeated `cnt` times. In other, every request repeated `cnt` times.
It seems that the second version is more visual.

I was surprised by the results of the script: the minimum was different for different `node_end_point` and with different `block_num_total`.
