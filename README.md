# sgf-parse - A library for parsing SGF files
A library for parsing [SGF FF\[4\]](https://www.red-bean.com/sgf/sgf4.html)
files in Rust.

`sgf-parse` provides a reliable but simple structured standard-compliate
interface for reading `.sgf` files. For all standard SGF data types, properties
are validated and parsed into appropriate native Rust types. Non-standard
properties are parsed and preserved.

[Documentation](https://docs.rs/sgf-parse)

## Installation
```
[dependencies]
sgf-parse = "0.2.1"
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to
discuss what you would like to change.
