use std::{env, sync::Arc, collections::HashMap};
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::Mutex};

struct Cache {
    data: HashMap<String, Vec<u8>>
}

impl Cache {
    fn set(self: &mut Self, key: String, data: &[u8]) {
        self.data.insert(key, data.to_vec());
    }

    fn get(&self, key: String) -> Option<&Vec<u8>> {
        self.data.get(&key)
    }

    fn exists(&self, key: String) -> bool {
        self.data.contains_key(&key)
    }

    fn delete(self: &mut Self, key: String) {
        self.data.remove(&key);
    }
}

#[derive(Debug)]
enum Command {
    QUIT = 1,
    GET = 2,
    SET = 3,
    EXISTS = 4,
    DELETE = 5,
    UNKNOWN = 255
}

impl Command {
    fn from_u8(c: u8) -> Command {
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

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut address = "127.0.0.1:9055".to_string();
    let mut max_key_size: usize = 32; // 32 b
    let mut max_value_size: usize = 1_024; // 1 kb

    if args.len() >= 4 {
        let addr_arg = &args[1];
        let key_arg = &args[2];
        let value_arg = &args[3];

        if addr_arg.len() == 0 || !addr_arg.contains(":") {
            panic!("{addr_arg} is not a valid value for listen address! must be in format of <hostname>:<port>. eg: 127.0.0.1:9055");
        }

        address = addr_arg.clone();

        match key_arg.parse::<usize>() {
            Ok(v) => max_key_size = v,
            Err(_) => panic!("{key_arg} is not a valid value for the max key size! must be in kb. eg: 1 = 1 b, 1000 = 1000 b (1 kb)")
        }

        match value_arg.parse::<usize>() {
            Ok(v) => max_value_size = v * 1_024,
            Err(_) => panic!("{value_arg} is not a valid value for the max value size! must be in kb. eg: 1 = 1 kb, 1000 = 1000 kb (1 mb)")
        }
    }

    let cache = Arc::new(Mutex::new(Cache {
        data: HashMap::new()
    }));

    let listener = TcpListener::bind(address.clone()).await?;

    println!("[i] listening on {}", address.clone());

    loop {
        let (mut socket, addr) = listener.accept().await?;

        println!("[+] {:?}", addr);

        tokio::spawn({
            let cache = cache.clone();

            async move {
                loop {
                    let mut buf = vec![0 as u8; 1 + max_key_size + max_value_size];
                    let n = socket.read(&mut buf).await?;

                    if n == 0 && buf[0] == 0 {
                        break;
                    }

                    let command = Command::from_u8(buf[0]);
                    let key = String::from_utf8_lossy(&buf[1..max_key_size + 1]).trim_end_matches(char::from(0)).to_string();
                    let data = &buf[max_key_size + 1..n];

                    match command {
                        Command::QUIT => {
                            println!("[-] {:?}", addr);

                            socket.shutdown().await?;
                        },
                        Command::GET => {
                            match cache.lock().await.get(key) {
                                Some(res) => socket.write_all(res).await?,
                                None => socket.write_all(&[0]).await?
                            }
                        },
                        Command::SET => {
                            cache.lock().await.set(key, data);
                            socket.write_all(&[1]).await?;
                        },
                        Command::EXISTS => {
                            let exists = cache.lock().await.exists(key);

                            socket.write_all(if exists { &[1] } else { &[0] }).await?;
                        },
                        Command::DELETE => {
                            cache.lock().await.delete(key);
                            socket.write_all(&[1]).await?;
                        },
                        Command::UNKNOWN => socket.write_all(&"UNKNOWN".as_bytes()).await?,
                    }
                }

                println!("[-] {:?}", addr);

                Ok::<(), io::Error>(())
            }
        });
    }
}