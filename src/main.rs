use std::{env};

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

    fn set(self: &mut Self, key: String, data: &[u8]) -> Result<(usize, usize), String> {
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

        Ok(range)
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
        mapping: vec![]
    };
}