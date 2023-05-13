use mr::{mapreduce::map_reduce_server::MapReduceServer, mr::coordinator::CoordinatorService};
use std::net::SocketAddr;
use std::thread;
use std::{env, sync::Arc};
use tonic::transport::Server;

// async fn start(
//     addr: std::net::SocketAddr,
//     service: CoordinatorService,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let svc = MapReduceServer::new(service);
//     Server::builder().add_service(svc).serve(addr).await?;
//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().count() < 2 {
        println!("Usage: mrcoordinator inputfiles...");
        std::process::exit(1);
    }

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_owned());
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    let files = env::args().skip(1).collect();
    let service = CoordinatorService::new(10, 10, files);
    let c = Arc::clone(&service.coordinator);

    println!("Coordinator listening on {}", addr);
    let handle = tokio::spawn(async move {
        let svc = MapReduceServer::new(service);
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .unwrap();
    });

    loop {
        if c.lock().unwrap().done() || handle.is_finished() {
            break;
        }
        // println!("zzzzzzzzz");
        thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
