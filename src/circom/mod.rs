//! # Circom Module
//!
//! This module is a slighly modified version of the original code from the <https://github.com/iden3/circom/> repository `circom` sub module, related to
//! the circom program structure, project configuration and compilation.

#![allow(clippy::result_unit_err)]

pub mod compilation;
pub mod execution;
pub mod input;
pub mod parser;
pub mod type_analysis;
