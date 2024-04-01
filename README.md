# Circom To Arithmetic Circuit

[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/eigen-trust/protocol/blob/master/LICENSE
[actions-badge]: https://github.com/eigen-trust/protocol/actions/workflows/test.yml/badge.svg
[actions-url]: https://github.com/eigen-trust/protocol/actions?query=branch%3Amaster

This library enables the creation of arithmetic circuits from circom programs.

Features supported:
|   |   |   |   |   |
|---|---|---|---|---|
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |

We aim to support canonical circomlib.
circomlib supported/modified:
|   |   |   |   |   |
|---|---|---|---|---|
|   |   |   |   |   |
|   |   |   |   |   |
|   |   |   |   |   |

Other important components such as circomlib-ml reside under apps.

## Requirements

- Rust: To install, follow the instructions found [here](https://www.rust-lang.org/tools/install).

## Getting Started

- Write your circom program in the `assets` directory under the `circuit.circom` name.

- Build the program

```bash
cargo build --release
```

- Run the compilation

```
cargo run
```

## Configuration

The CLI provides options to specify both input and output file paths, allowing flexibility in how you manage your circom program files and the resulting arithmetic circuits.

### Command Line Arguments

- `--input`: Specifies the path to the input circom program file. By default, the program looks for `circuit.circom` in the `assets` directory.
- `--output`: Specifies the path to the output directory where the generated arithmetic circuit files will be stored. By default, it's saved in the `./output` directory.

#### Example

To run the program with specific input file path and output directory path, use the following command format:

```bash
./target/release/circom --input ./input-path/circuit.circom --output ./output-path/
```

## Contributing

Contributions are welcome!

## License

This project is licensed under the MIT License - see the LICENSE file for details.
