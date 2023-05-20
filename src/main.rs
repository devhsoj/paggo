use paggo::PaggoInstance;
use std::{env, sync::Arc};
use tokio::io;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();

    let mut port = 9055;
    let mut max_key_size: usize = 32; // default: 32 b
    let mut max_value_size: usize = 1_024; // default: 1 kb

    if args.len() >= 4 {
        let port_arg = &args[1];
        let key_arg = &args[2];
        let value_arg = &args[3];

        match port_arg.parse() {
            Ok(p) => {
                port = p;
            }
            Err(_) => panic!("{port_arg} is not a valid value for listen address! must be in the format of <hostname>:<port>. eg: 127.0.0.1:9055")
        }

        match key_arg.parse::<usize>() {
            Ok(v) => max_key_size = v,
            Err(_) => panic!("{key_arg} is not a valid value for the max key size! must be in bytes. eg: 1 (would be 1 b)")
        }

        match value_arg.parse::<usize>() {
            Ok(v) => max_value_size = v,
            Err(_) => panic!("{value_arg} is not a valid value for the max value size! must be in bytes. eg: 1024 (would be 1 kb)")
        }
    }
    println!("[i] listening on {}", port);
    Arc::new(PaggoInstance::new(port, max_key_size, max_value_size))
        .run()
        .await
}
