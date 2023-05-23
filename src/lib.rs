//! # Paggo
//!
//! This crate exports [Paggo], allowing the creation custom instances for it and integrating it on existing services.

use std::{io, net::Ipv6Addr, sync::Arc, mem::transmute};

use dashmap::DashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

const SUCCESS: u8 = 1;
const ERROR: u8 = 0;

/// Runs an instance of the Paggo database with the specified settings.
#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(clap::Parser))]
#[cfg_attr(feature = "cli", command(author = "devhsoj", version = env!("CARGO_PKG_VERSION"), about = "A simple database server implementation.", long_about = None))]
pub struct PaggoInstance {
    #[cfg_attr(feature = "cli", arg(default_value_t = 9055))]
    pub(crate) port: u16,
    #[cfg_attr(feature = "cli", arg(default_value_t = 32))]
    pub(crate) max_key_size: usize,
    #[cfg_attr(feature = "cli", arg(default_value_t = 1024))]
    pub(crate) max_value_size: usize,
}

impl PaggoInstance {
    /// Creates a new [`PaggoInstance`]. Takes the port Paggo should run on, the maximum key length, and the maximum
    /// value length.
    pub fn new(port: u16, max_key_size: usize, max_value_size: usize) -> Self {
        Self {
            port,
            max_key_size,
            max_value_size,
        }
    }

    /// Runs the [`PaggoInstance`] until an error is encountered.
    pub async fn run(self: &Arc<Self>) -> Result<(), io::Error> {
        let listener =
            TcpListener::bind((Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), self.port)).await?;
        let cache = DashMap::<String, Vec<u8>>::new();
        let key_start = if self.max_key_size == 1 {
            self.max_key_size
        } else {
            self.max_key_size + 1
        };

        loop {
            let (mut socket, _) = listener.accept().await?;
            let cache = cache.clone();
            let self_ref = self.clone();

            tokio::spawn({
                async move {
                    loop {
                        let mut buf =
                            vec![0u8; 1 + self_ref.max_key_size + self_ref.max_value_size];
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
                            Command::GET => match cache.get(&key) {
                                Some(res) => socket.write_all(res.value()).await?,
                                None => socket.write_all(&[ERROR]).await?,
                            },
                            Command::SET => {
                                cache.insert(key, data.to_vec());
                                socket.write_all(&[SUCCESS]).await?;
                            }
                            Command::EXISTS => {
                                socket.write_all(&[cache.contains_key(&key) as u8]).await?;
                            }
                            Command::DELETE => {
                                cache.remove(&key);
                                socket.write_all(&[SUCCESS]).await?;
                            }
                            Command::PING => {
                                socket.write_all(&[SUCCESS]).await?;
                            }
                            Command::QUIT => {
                                socket.shutdown().await?;
                            }
                            Command::UNKNOWN => socket.write_all(&[ERROR]).await?,
                        }
                    }
                    Ok::<(), io::Error>(())
                }
            });
        }
    }
}

/// Represents a command that can be given to Paggo.
#[repr(u8)]
#[derive(Debug)]
pub enum Command {
    QUIT = 1,
    GET = 2,
    SET = 3,
    EXISTS = 4,
    DELETE = 5,
    PING = 6,
    UNKNOWN = 255,
}

impl Command {
    pub fn from_u8(c: u8) -> Command {
        if c <= 6 { // with rust nightly this would be `c < std::mem::variant_count::<Self>()`
            unsafe { transmute(c) }
        } else {
            Self::UNKNOWN
        }
    }
}
