# sgf-parse - A library for parsing SGF files

![Continuous integration](https://github.com/julianandrews/sgf-parse/workflows/Continuous%20integration/badge.svg)

A library for parsing [SGF FF\[4\]](https://www.red-bean.com/sgf/sgf4.html)
files in Rust.

`sgf-parse` provides a reliable but simple structured, standard-compliant
interface for reading and writing `.sgf` files. For all standard SGF data types,
properties are validated and parsed into appropriate native Rust types.
Non-standard properties are parsed and preserved.

[Documentation](https://docs.rs/sgf-parse)

## Installation
Find `sgf-parse` on [crates.io](https://crates.io/crates/sgf-parse)

## Contributing
Pull requests are welcome. For major changes, please open an issue first to
discuss what you would like to change.

I would be particularly interested in any PRs to add support for non-Go games.
Right now `sgf-parse` in principle can support any games supported by SGF, but
I've only got specific implementations for Go, and a catchall with no special
behavior where moves, stones, and points are just strings left to the library
user to interpret.
