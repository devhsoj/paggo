use clap::Parser;
use paggo::PaggoInstance;
use std::sync::Arc;
use tokio::io;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    Arc::new(PaggoInstance::parse()).run().await
}

