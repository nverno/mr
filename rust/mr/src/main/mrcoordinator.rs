use mr::mr::coordinator::{mapreduce::map_reduce_server::MapReduceServer, CoordinatorService};
use std::env;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().count() < 2 {
        println!("Usage: mrcoordinator inputfiles...");
        std::process::exit(1);
    }

    let addr = "[::1]:10000".parse().unwrap();

    let c = CoordinatorService::new(10);

    let svc = MapReduceServer::new(c);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
