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

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_owned());
    let addr = format!("127.0.0.1:{}", port).parse().unwrap();

    let files = env::args().skip(1).collect();
    let c = CoordinatorService::new(10, 10, files);

    let svc = MapReduceServer::new(c);

    println!("Coordinator listening on {}", addr);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
