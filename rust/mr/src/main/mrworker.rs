pub mod mapreduce {
    tonic::include_proto!("mapreduce");
}
use mapreduce::map_reduce_client::MapReduceClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().count() != 2 {
        println!("Usage: mrworker xxx.so");
        std::process::exit(1);
    }

    // TODO: start a worker process
    // - load map/reduce functions from plugin
    // - start work
    let mut client = MapReduceClient::connect("http://[::1]:10000").await?;

    Ok(())
}
