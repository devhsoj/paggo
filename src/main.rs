use std::{env, sync::Arc};
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::Mutex};

struct Cache {
    buffer: Vec<u8>,
    mapping: Vec<(String, (usize, usize))>
}

impl Cache {
    fn next_index(self: &Self) -> usize {
        let keys_length = self.mapping.len();

        if keys_length == 0 {
            return 0;
        }

        let (_, range) = &self.mapping[keys_length - 1];

        range.1
    }

    fn set(self: &mut Self, key: String, data: &[u8]) -> Result<(), String> {
        let data_length = data.len();
        let mut start = self.next_index();

        let exists = self.exists(key.clone());

        if exists.is_some() {
            start = exists.unwrap().1;
        }

        let buffer_length = self.buffer.len();

        if start + data_length > buffer_length {
            return Err(format!(
                "Not enough memory. Tried storing {} b with only {} b left!",
                data_length,
                buffer_length - start
            ));
        }

        self.buffer[start..start + data_length].clone_from_slice(data);

        let end = start + data_length;
        let range = (start, end);

        self.mapping.push((key, range));

        Ok(())
    }

    fn get(self: &Self, key: String) -> Option<&[u8]> {
        let mut range: Option<&(usize, usize)> = None;

        for i in 0..self.mapping.len() {
            if key == self.mapping[i].0 {
                range = Some(&self.mapping[i].1);
            }
        }

        if range.is_none() {
            return None;
        }

        let indexes = range.unwrap();

        Some(&self.buffer[indexes.0..indexes.1])
    }

    fn exists(self: &Self, key: String) -> Option<&(usize, usize)> {
        for i in 0..self.mapping.len() {
            if key == self.mapping[i].0 {
                return Some(&self.mapping[i].1);
            }
        }

        None
    }
}

#[derive(Debug)]
enum Command {
    QUIT = 1,
    GET = 2,
    SET = 3,
    EXISTS = 4,
    UNKNOWN = 255
}

impl Command {
    fn from_u8(c: u8) -> Command {
        match c {
            1 => Command::QUIT,
            2 => Command::GET,
            3 => Command::SET,
            4 => Command::EXISTS,
            _ => Command::UNKNOWN,
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut allocated: usize = 1_024 * 1_024; // 1 mb
    let mut max_key_size: usize = 32; // 32 b
    let mut max_value_size: usize = 1_024; // 1 kb

    if args.len() >= 3 {
        let alloc_arg = &args[1];
        let key_arg = &args[2];
        let value_arg = &args[3];

        match alloc_arg.parse::<usize>() {
            Ok(v) => allocated = v * 1_024 * 1_024,
            Err(_) => println!("{} is not a valid value for allocated memory! must be in mb. eg: 1 = 1 mb, 1000 = 1,000 mb (1 gb)", alloc_arg)
        }

        match key_arg.parse::<usize>() {
            Ok(v) => max_key_size = v,
            Err(_) => println!("{} is not a valid value for the max key size! must be in kb. eg: 1 = 1 b, 1000 = 1000 b (1 kb)", alloc_arg)
        }

        match value_arg.parse::<usize>() {
            Ok(v) => max_value_size = v * 1_024,
            Err(_) => println!("{} is not a valid value for the max value size! must be in kb. eg: 1 = 1 kb, 1000 = 1000 kb (1 mb)", alloc_arg)
        }
    }

    let cache = Arc::new(Mutex::new(Cache {
        buffer: vec![0; allocated],
        mapping: vec![]
    }));

    let listener = TcpListener::bind("127.0.0.1:9055").await?;

    println!("[i] listening on localhost:9055");

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
                    let data = &buf[max_key_size + 1..];

                    match command {
                        Command::QUIT => {
                            println!("[-] {:?}", addr);

                            socket.shutdown().await?;
                        },
                        Command::GET => {
                            match &mut cache.lock().await.get(key) {
                                Some(res) => socket.write_all(res).await?,
                                None => socket.write_all(&[0]).await?
                            }
                        },
                        Command::SET => {
                            match &mut cache.lock().await.set(key, data) {
                                Ok(_) => socket.write_all(&[1]).await?,
                                Err(e) => socket.write_all(e.as_bytes()).await?
                            }
                        },
                        Command::EXISTS => {
                            match &mut cache.lock().await.exists(key) {
                                Some(_) => socket.write_all(&[1]).await?,
                                None => socket.write_all(&[0]).await?
                            }
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