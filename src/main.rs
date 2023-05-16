use std::{env, collections::HashMap};

struct Cache {
    buffer: Vec<u8>,
    mapping: HashMap<String, (usize, usize)>
}

impl Cache {
    fn next_index(self: &Self) -> usize {
        let keys: Vec<&String> = self.mapping.keys().map(|key| key).collect();
        let keys_length = keys.len();

        if keys_length == 0 {
            return 0;
        }

        let last_key = keys[keys_length - 1];

        self.mapping.get(last_key).expect("Failed to retrieve last key!").1
    }

    fn set(self: &mut Self, key: String, data: &Vec<u8>) -> Result<(usize, usize), String> {
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

        for i in 0..data_length {
            self.buffer[start + i] = data[i];
        }

        let end = start + data_length;
        let range = (start, end);

        self.mapping.insert(key, range);

        Ok(range)
    }

    fn get(self: &Self, key: String) -> Option<&[u8]> {
        let range_entry = self.mapping.get(&key);

        if range_entry.is_none() {
            return None;
        }

        let range = range_entry.unwrap();

        Some(&self.buffer[range.0..range.1])
    }

    fn exists(self: &Self, key: String) -> Option<&(usize, usize)> {
        self.mapping.get(&key)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut allocated: usize = 0;

    if args.len() < 2 {
        allocated = 1 * 1_024 * 1_024; // 1 mb
    } else {
        let memory_arg = &args[1];

        match memory_arg.parse::<usize>() {
            Ok(v) => allocated = v * 1_024 * 1_024,
            Err(_) => println!("{} is not a valid memory value! must be in mb", memory_arg)
        }
    }

    let mut cache = Cache {
        buffer: vec![0; allocated],
        mapping: HashMap::new()
    };
}