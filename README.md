# Circom To Arithmetic Circuit

[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/eigen-trust/protocol/blob/master/LICENSE
[actions-badge]: https://github.com/eigen-trust/protocol/actions/workflows/test.yml/badge.svg
[actions-url]: https://github.com/eigen-trust/protocol/actions?query=branch%3Amaster

This library enables the creation of arithmetic circuits from circom programs.

## Supported Circom Features

| Category        | Type                     | Supported |
| --------------- | ------------------------ | :-------: |
| **Statements**  | `InitializationBlock`    |    ✅     |
|                 | `Block`                  |    ✅     |
|                 | `Substitution`           |    ✅     |
|                 | `Declaration`            |    ✅     |
|                 | `IfThenElse`             |    ✅     |
|                 | `While`                  |    ✅     |
|                 | `Return`                 |    ✅     |
|                 | `MultSubstitution`       |    ❌     |
|                 | `UnderscoreSubstitution` |    ❌     |
|                 | `ConstraintEquality`     |    ❌     |
|                 | `LogCall`                |    ❌     |
|                 | `Assert`                 |    ❌     |
| **Expressions** | `Call`                   |    ✅     |
|                 | `InfixOp`                |    ✅     |
|                 | `Number`                 |    ✅     |
|                 | `Variable`               |    ✅     |
|                 | `PrefixOp`               |    ✅     |
|                 | `InlineSwitchOp`         |    ❌     |
|                 | `ParallelOp`             |    ❌     |
|                 | `AnonymousComp`          |    ❌     |
|                 | `ArrayInLine`            |    ❌     |
|                 | `Tuple`                  |    ❌     |
|                 | `UniformArray`           |    ❌     |

## Circomlib

WIP

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
cargo run --release
```

The compiled circuit and circuit report can be found in the `./output` directory.

## ZK/MPC/FHE backends:

- [2PC-GC with mpz-bmr16](https://github.com/tkmct/mpz/tree/bmr16)
- [MP-SPDZ MPC](https://github.com/mhchia/MP-SPDZ/tree/arith-executor)
- [TFHE-rs](https://github.com/namnc/circom-thfe-rs)

## Contributing

Contributions are welcome!

## License

This project is licensed under the MIT License - see the LICENSE file for details.
