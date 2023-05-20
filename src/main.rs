mod command;

use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::TcpListener};
use std::{env, sync::Arc};
use dashmap::DashMap;
use command::Command;

fn decode_key(buf: &[u8]) -> String {
    String::from_utf8_lossy(&buf).trim_end_matches(char::from(0)).to_string()
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut address = "127.0.0.1:9055".to_string();
    let mut max_key_size: usize = 32; // default: 32 b
    let mut max_value_size: usize = 1_024; // default: 1 kb

    let mut key_start = max_key_size + 1;

    if args.len() >= 4 {
        let addr_arg = &args[1];
        let key_arg = &args[2];
        let value_arg = &args[3];

        if addr_arg.len() == 0 || !addr_arg.contains(":") {
            panic!("{addr_arg} is not a valid value for listen address! must be in the format of <hostname>:<port>. eg: 127.0.0.1:9055");
        }

        address = addr_arg.clone();

        match key_arg.parse::<usize>() {
            Ok(v) => max_key_size = v,
            Err(_) => panic!("{key_arg} is not a valid value for the max key size! must be in bytes. eg: 1 (would be 1 b)")
        }

        if max_key_size == 1 {
            key_start = 1;
        }

        match value_arg.parse::<usize>() {
            Ok(v) => max_value_size = v,
            Err(_) => panic!("{value_arg} is not a valid value for the max value size! must be in bytes. eg: 1024 (would be 1 kb)")
        }
    }

    let cache: Arc<DashMap<String, Vec<u8>>> = Arc::new(DashMap::new());
    let listener = TcpListener::bind(address.clone()).await?;

    println!("[i] listening on {}", address.clone());

    loop {
        let (mut socket, addr) = listener.accept().await?;

        println!("[+] {:?}", addr);

        tokio::spawn({

            let cache = Arc::clone(&cache);

            async move {
                loop {
                    let mut buf = vec![0 as u8; 1 + max_key_size + max_value_size];
                    let n = socket.read(&mut buf).await?;

                    if n == 0 && buf[0] == 0 {
                        break;
                    }

                    let command = Command::from_u8(buf[0]);

                    let res = match command {
                        Command::PING => {
                            socket.write_all(&[1]).await
                        },
                        Command::QUIT => {
                            println!("[-] {:?}", addr);

                            socket.shutdown().await
                        },
                        Command::GET => {
                            let key = decode_key(&buf[1..max_key_size + 1]);

                            match cache.get(&key) {
                                Some(res) => socket.write_all(&res).await,
                                None => socket.write_all(&[0]).await
                            }
                        },
                        Command::SET => {
                            let key = decode_key(&buf[1..max_key_size + 1]);
                            let data = &buf[key_start..n];

                            cache.insert(key, data.to_vec());
                            socket.write_all(&[1]).await
                        },
                        Command::EXISTS => {
                            let key = decode_key(&buf[1..max_key_size + 1]);
                            let exists = cache.contains_key(&key);

                            socket.write_all(if exists { &[1] } else { &[0] }).await
                        },
                        Command::DELETE => {
                            let key = decode_key(&buf[1..max_key_size + 1]);

                            cache.remove(&key);
                            socket.write_all(&[1]).await
                        },
                        Command::UNKNOWN => socket.write_all(&"UNKNOWN".as_bytes()).await,
                    };

                    match res {
                        Ok(()) => (),
                        Err(e) => {
                            eprintln!("{}", e.to_string());

                            socket.write_all(&[0]).await?;
                            socket.shutdown().await?;
                        }
                    }
                }

                println!("[-] {:?}", addr);

                Ok::<(), io::Error>(())
            }
        });
    }
}