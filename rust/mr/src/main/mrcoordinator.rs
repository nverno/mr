use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().count() < 2 {
        println!("Usage: mrcoordinator inputfiles...");
        std::process::exit(1);
    }

    // TODO: init coordinator and loop till all tasks are done
    Ok(())
}
