# Circom To Arithmetic Circuit

[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![codecov](https://codecov.io/github/namnc/circom-2-arithc/graph/badge.svg)](https://app.codecov.io/github/namnc/circom-2-arithc/)

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/namnc/circom-2-arithc/blob/master/LICENSE
[actions-badge]: https://github.com/namnc/circom-2-arithc/actions/workflows/build.yml/badge.svg
[actions-url]: https://github.com/namnc/circom-2-arithc/actions?query=branch%3Amain

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
|                 | `Assert`                 |    ✅     |
| **Expressions** | `Call`                   |    ✅     |
|                 | `InfixOp`                |    ✅     |
|                 | `Number`                 |    ✅     |
|                 | `Variable`               |    ✅     |
|                 | `PrefixOp`               |    ✅     |
|                 | `InlineSwitchOp`         |    ❌     |
|                 | `ParallelOp`             |    ❌     |
|                 | `AnonymousComp`          |    ✅     |
|                 | `ArrayInLine`            |    ❌     |
|                 | `Tuple`                  |    ✅     |
|                 | `UniformArray`           |    ❌     |

## Circomlib

WIP

## Requirements

- Rust: To install, follow the instructions found [here](https://www.rust-lang.org/tools/install).

## Getting Started

- Write your circom program in the `input` directory under the `circuit.circom` name.

- Build the program

```bash
cargo build --release
```

- Run the compilation

```bash
cargo run --release
```

The compiled circuit and circuit report can be found in the `./output` directory.

### Boolean Circuits

Although this library is named after arithmetic circuits, the CLI integrates [boolify](https://github.com/voltrevo/boolify) allowing further compilation down to boolean circuits.

To achieve this, add `--boolify-width DESIRED_INT_WIDTH` to your command:

```bash
cargo run --release -- --boolify-width 16
```

## ZK/MPC/FHE backends:

- [circom-mp-spdz](https://github.com/namnc/circom-mp-spdz)

## Contributing

Contributions are welcome!

## License

This project is licensed under the MIT License - see the LICENSE file for details.
