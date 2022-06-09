# Script to define optimal batch size and concurrency of blocks import

## Run
Pass the arguments in command line:
```
RUST_LOG=info cargo run node_end_point block_num_total cnt
```
Where:  
- `RUST_LOG=info` is for tracking errors in requests (optional)  
- `node_end_point` is node for test (e. g. *https://rpc.xdaichain.com/*)  
- `block_num_total` is number of generated blocks  
- `cnt` is number of runs (optional, 10 by default)  

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
csv = "1.1.6"
anyhow = "1.0.57"
```
## Structure of concurrency 
The picture shows how the block numbers are stored in memory and how concurrency is applied to them:

![](https://i.imgur.com/qCOH6eB.png)

## Analysis
Let's take a look at the distribution when iterating `block_batch_size`:

* These plots shows dependence of time on concurrency. Vertical line is the num of cores (I have 8).
![1](https://user-images.githubusercontent.com/70902141/171616230-9d7a71d2-4e7d-4aec-a914-0f47ebb9bce9.png)
![2](https://user-images.githubusercontent.com/70902141/171616234-e6124ca4-7e7c-466e-adfa-1c447f3d8ed3.png)
As we can see, when script create more than 8 green threads, the scheduler makes a big contribution to performance.  
It also applies to *eth_getTransactionReceipt* request:  
![image](https://user-images.githubusercontent.com/70902141/171616633-824e80ec-f040-4b30-b00a-c0e94dfe29a7.png)
* These graphs plotted for https://sokol.poa.network/ node.
![4](https://user-images.githubusercontent.com/70902141/171633675-5038c5e4-efe7-45d7-a97c-ea5e528e04c4.png)
![image](https://user-images.githubusercontent.com/70902141/171634703-d572b995-66ed-4b29-adb8-bd6ddeb6e4cc.png)
Analyzing them, we can put forward a hypothesis about the best enumeration of variables.  
One of the hypothesis is: *change varible `block_concurrency`, thus, go by divisors of `block_num_total`*.
* Here graph for https://rpc.xdaichain.com/ node.
![image](https://user-images.githubusercontent.com/70902141/171635106-ccf2ead1-10ff-40fd-800e-aa50df5d18c3.png)
For *eth_getBlockByNumber requests* we can see two other minimums, not only (10, 4). There are (7, 6) and (15, 3).

  
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
