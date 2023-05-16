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

    fn get(self: &Self, key: String) -> Option<Vec<u8>> {
        let range_entry = self.mapping.get(&key);

        if range_entry.is_none() {
            return None;
        }

        let range = range_entry.unwrap();
        let mut vec: Vec<u8> = Vec::new();

        for c in &self.buffer[range.0..range.1] {
            vec.push(*c);
        }

        Some(vec)
    }

    fn exists(self: &Self, key: String) -> Option<&(usize, usize)> {
        self.mapping.get(&key)
    }

}

fn main() {

}