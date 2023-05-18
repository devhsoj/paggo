use std::{env, sync::Arc};
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::Mutex};

struct Cache {
    buffer: Vec<u8>,
    mapping: Vec<(String, (usize, usize))>
}

impl Cache {
    fn get_free_range(self: &Self, size: usize) -> (usize, usize) {
        let mut range: (usize, usize) = (0, size);

        for entry in &self.mapping {
            let entry_range = entry.1;

            if (range.0 > entry_range.0 && range.1 < entry_range.1)
            || (range.0 > entry_range.0 && range.1 > entry_range.1) {
                continue;
            }

            range.0 = entry_range.1;
            range.1 = entry_range.1 + size;
        }

        range
    }

    async fn set(self: &mut Self, key: String, data: &[u8]) -> io::Result<()> {
        if self.exists(key.clone()).is_some() {
            self.clear(key.clone());
        }

        let data_length = data.len();
        let free_range = self.get_free_range(data_length);
        let buffer_length = self.buffer.len();

        if free_range.1 + data_length > buffer_length {
            return Err(io::Error::new(io::ErrorKind::OutOfMemory, format!(
                "Not enough memory. Tried storing {} b with only {} b left!",
                data_length,
                buffer_length - free_range.1
            )));
        }

        for i in 0..data_length {
            (&mut self.buffer)[free_range.0 + i] = data[i];
        }

        self.mapping.push((key, free_range));

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

    fn clear(self: &mut Self, key: String) {
        let mut range: Option<(usize, usize)> = None;

        for i in 0..self.mapping.len() {
            if key == self.mapping[i].0 {
                range = Some(self.mapping[i].1);
                self.mapping.remove(i);
                break;
            }
        }

        if range.is_none() {
            return;
        }

        let range = range.unwrap();

        self.buffer[range.0..range.1].clone_from_slice(&vec![0; range.1 - range.0]);
    }
}

#[derive(Debug)]
enum Command {
    QUIT = 1,
    GET = 2,
    SET = 3,
    EXISTS = 4,
    CLEAR = 5,
    UNKNOWN = 255
}

impl Command {
    fn from_u8(c: u8) -> Command {
        match c {
            1 => Command::QUIT,
            2 => Command::GET,
            3 => Command::SET,
            4 => Command::EXISTS,
            5 => Command::CLEAR,
            _ => Command::UNKNOWN,
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut address = "127.0.0.1:9055".to_string();
    let mut allocated: usize = 1_024 * 1_024; // 1 mb
    let mut max_key_size: usize = 32; // 32 b
    let mut max_value_size: usize = 1_024; // 1 kb

    if args.len() >= 4 {
        let addr_arg = &args[1];
        let alloc_arg = &args[2];
        let key_arg = &args[3];
        let value_arg = &args[4];

        if addr_arg.len() == 0 || !addr_arg.contains(":") {
            panic!("{addr_arg} is not a valid value for listen address! must be in format of <hostname>:<port>. eg: 127.0.0.1:9055");
        }

        address = addr_arg.clone();

        match alloc_arg.parse::<usize>() {
            Ok(v) => allocated = v * 1_024 * 1_024,
            Err(_) => panic!("{alloc_arg} is not a valid value for allocated memory! must be in mb. eg: 1 = 1 mb, 1000 = 1,000 mb (1 gb)")
        }

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
        buffer: vec![0; allocated],
        mapping: vec![]
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
                            match &mut cache.lock().await.get(key) {
                                Some(res) => socket.write_all(res).await?,
                                None => socket.write_all(&[0]).await?
                            }
                        },
                        Command::SET => {
                            match &mut cache.lock().await.set(key, data).await {
                                Ok(_) => socket.write_all(&[1]).await?,
                                Err(e) => socket.write_all(e.to_string().as_bytes()).await?
                            }
                        },
                        Command::EXISTS => {
                            match &mut cache.lock().await.exists(key) {
                                Some(_) => socket.write_all(&[1]).await?,
                                None => socket.write_all(&[0]).await?
                            }
                        },
                        Command::CLEAR => {
                            let _ = &mut cache.lock().await.clear(key);
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