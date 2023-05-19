# paggo

A small, fast, and safe in-memory database. 

## Installation

```bash
git clone https://github.com/devhsoj/paggo.git
cargo build --release
```

## Example Usage

```bash
# ./path/to/paggo [listen address] [max key size] [max value size]

# listens on 127.0.0.1:9055, max key size: 32 b, max value size: 1 kb
./path/to/paggo

# listens on 0.0.0.0:3333, max key size: 256 b, max value size: 1 mb
./path/to/paggo 0.0.0.0:3333 256 1000
```

## License

[MIT](https://choosealicense.com/licenses/mit/)