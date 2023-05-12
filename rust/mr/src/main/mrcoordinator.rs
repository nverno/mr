// use crate::mapreduce::{mapreduce::map_reduce_server::MapReduceServer, CoordinatorService};
use std::env;
use mr::{mr::coordinator::CoordinatorService, mapreduce::map_reduce_server::MapReduceServer};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().count() < 2 {
        println!("Usage: mrcoordinator inputfiles...");
        std::process::exit(1);
    }

    let addr = "http://[::1]:10000".parse().unwrap();

    let files = env::args().skip(1).collect();
    let c = CoordinatorService::new(10, 10, files);

    let svc = MapReduceServer::new(c);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
