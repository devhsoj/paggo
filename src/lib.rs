use std::{collections::HashMap, io, net::Ipv6Addr, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
};

/// # Paggo
///
/// This crate exports [Paggo], allowing creating custom instances for it and integrating it on existing services.

pub struct PaggoInstance {
    port: u16,
    max_key_size: usize,
    max_value_size: usize,
}

impl PaggoInstance {
    pub fn new(port: u16, max_key_size: usize, max_value_size: usize) -> Self {
        Self {
            port,
            max_key_size,
            max_value_size,
        }
    }

    pub async fn run(self: &Arc<Self>) -> Result<(), io::Error> {
        let listener =
            TcpListener::bind((Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), self.port)).await?;
        let cache: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));
        let key_start = if self.max_key_size == 1 {
            self.max_key_size
        } else {
            self.max_key_size + 1
        };

        loop {
            let (mut socket, addr) = listener.accept().await?;
            let cache = cache.clone();
            let self_ref = self.clone();

            tokio::spawn({
                async move {
                    loop {
                        let mut buf =
                            vec![0 as u8; 1 + self_ref.max_key_size + self_ref.max_value_size];
                        let n = socket.read(&mut buf).await?;

                        if n == 0 && buf[0] == 0 {
                            break;
                        }

                        let command = Command::from_u8(buf[0]);
                        let key = String::from_utf8_lossy(&buf[1..self_ref.max_key_size + 1])
                            .trim_end_matches(char::from(0))
                            .to_string();
                        let data = &buf[key_start..n];

                        match command {
                            Command::QUIT => {
                                println!("[-] {:?}", addr);

                                socket.shutdown().await?;
                            }
                            Command::GET => match cache.lock().await.get(&key) {
                                Some(res) => socket.write_all(res).await?,
                                None => socket.write_all(&[0]).await?,
                            },
                            Command::SET => {
                                cache.lock().await.insert(key, data.to_vec());
                                socket.write_all(&[1]).await?;
                            }
                            Command::EXISTS => {
                                let exists = cache.lock().await.contains_key(&key);

                                socket.write_all(if exists { &[1] } else { &[0] }).await?;
                            }
                            Command::DELETE => {
                                cache.lock().await.remove(&key);
                                socket.write_all(&[1]).await?;
                            }
                            Command::UNKNOWN => socket.write_all(&"UNKNOWN".as_bytes()).await?,
                        }
                    }

                    println!("[-] {:?}", addr);

                    Ok::<(), io::Error>(())
                }
            });
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum Command {
    QUIT = 1,
    GET = 2,
    SET = 3,
    EXISTS = 4,
    DELETE = 5,
    UNKNOWN = 255,
}

impl Command {
    pub fn from_u8(c: u8) -> Command {
        match c {
            1 => Command::QUIT,
            2 => Command::GET,
            3 => Command::SET,
            4 => Command::EXISTS,
            5 => Command::DELETE,
            _ => Command::UNKNOWN,
        }
    }
}
