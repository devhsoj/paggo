#![cfg_attr(feature = "cli", allow(unused_imports))]
#[cfg(feature = "cli")]
use clap::Parser;
use paggo::PaggoInstance;
use std::{sync::Arc, process::exit};
use tokio::io;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    #[cfg(not(feature = "cli"))]
    {
        eprintln!("The CLI program is disabled!");
        exit(1)
    }

    #[cfg(feature = "cli")]
    Arc::new(PaggoInstance::parse()).run().await
}
