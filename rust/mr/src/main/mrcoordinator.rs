// use crate::mapreduce::{mapreduce::map_reduce_server::MapReduceServer, CoordinatorService};
use mr::{mapreduce::map_reduce_server::MapReduceServer, mr::coordinator::CoordinatorService};
use std::env;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().count() < 2 {
        println!("Usage: mrcoordinator inputfiles...");
        std::process::exit(1);
    }

    let addr = env::var("COORD_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_owned())
        .parse()
        .unwrap();

    let files = env::args().skip(1).collect();
    let c = CoordinatorService::new(10, 10, files);

    let svc = MapReduceServer::new(c);

    println!("Coordinator listening on {}", addr);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
