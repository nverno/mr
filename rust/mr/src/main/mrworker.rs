use libloading::{Library, Symbol};
use mr::{mapreduce::map_reduce_client::MapReduceClient, mr::worker::work, MapFunc, ReduceFunc};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = env::args().nth(1).expect("Usage: mr-worker xxx.so");

    unsafe {
        // Load map/reduce functions from dynamic library
        let lib = Library::new(lib).expect("failed to open shared library");
        let mapf: Symbol<MapFunc> = lib.get(b"map").expect("failed to load map function");

        let reducef: Symbol<ReduceFunc> =
            lib.get(b"reduce").expect("failed to load reduce function");

        let mut w = MapReduceClient::connect(
            env::var("COORD_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned()),
        )
        .await?;

        work(&mut w, *mapf, *reducef).await?
    }

    Ok(())
}
