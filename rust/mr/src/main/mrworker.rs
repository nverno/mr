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

    Ok(())
}
