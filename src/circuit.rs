//! # Circuit Module
//!
//! This module defines structures and operations for arithmetic circuits, including variables, gates, and circuit composition.

use circom_program_structure::ast::ExpressionInfixOpcode;
use mpz_circuits::{BuilderError, GateType};
use regex::Captures;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

/// Types of gates that can be used in an arithmetic circuit.
#[derive(Serialize, Deserialize, Debug)]
pub enum AGateType {
    AAdd,
    ADiv,
    AEq,
    AGEq,
    AGt,
    ALEq,
    ALt,
    AMul,
    ANeq,
    ANone,
    ASub,
}

impl Display for AGateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AGateType::ANone => write!(f, "None"),
            AGateType::AAdd => write!(f, "AAdd"),
            AGateType::ASub => write!(f, "ASub"),
            AGateType::AMul => write!(f, "AMul"),
            AGateType::ADiv => write!(f, "ADiv"),
            AGateType::AEq => write!(f, "AEq"),
            AGateType::ANeq => write!(f, "ANEq"),
            AGateType::ALEq => write!(f, "ALEq"),
            AGateType::AGEq => write!(f, "AGEq"),
            AGateType::ALt => write!(f, "ALt"),
            AGateType::AGt => write!(f, "AGt"),
        }
    }
}

impl From<&ExpressionInfixOpcode> for AGateType {
    fn from(opcode: &ExpressionInfixOpcode) -> Self {
        match opcode {
            ExpressionInfixOpcode::Add => AGateType::AAdd,
            ExpressionInfixOpcode::Div => AGateType::ADiv,
            ExpressionInfixOpcode::Eq => AGateType::AEq,
            ExpressionInfixOpcode::Greater => AGateType::AGt,
            ExpressionInfixOpcode::GreaterEq => AGateType::AGEq,
            ExpressionInfixOpcode::Lesser => AGateType::ALt,
            ExpressionInfixOpcode::LesserEq => AGateType::ALEq,
            ExpressionInfixOpcode::Mul => AGateType::AMul,
            ExpressionInfixOpcode::NotEq => AGateType::ANeq,
            ExpressionInfixOpcode::Sub => AGateType::ASub,
            _ => AGateType::ANone,
        }
    }
}

/// Represents a variable in an arithmetic circuit, with optional constant value properties.
#[derive(Serialize, Deserialize, Debug)]
pub struct ArithmeticVar {
    pub var_id: u32,
    pub var_name: String,
    pub is_const: bool,
    pub const_value: u32,
}

impl ArithmeticVar {
    pub fn new(_var_id: u32, _var_name: String) -> ArithmeticVar {
        ArithmeticVar {
            var_id: _var_id,
            var_name: _var_name,
            is_const: false,
            const_value: 0,
        }
    }

    pub fn set_const_value(&mut self, value: u32) {
        self.is_const = true;
        self.const_value = value;
    }
}

/// Defines a gate in an arithmetic circuit, including its type and input/output variable IDs.
#[derive(Serialize, Deserialize, Debug)]
pub struct ArithmeticNode {
    pub gate_id: u32,
    pub gate_type: AGateType,
    pub input_lhs_id: u32,
    pub input_rhs_id: u32,
    pub output_id: u32,
}

impl ArithmeticNode {
    pub fn new(
        _gate_id: u32,
        _gate_type: AGateType,
        _input_lhs_id: u32,
        _input_rhs_id: u32,
        _out_put_id: u32,
    ) -> ArithmeticNode {
        ArithmeticNode {
            gate_id: _gate_id,
            gate_type: _gate_type,
            input_lhs_id: _input_lhs_id,
            input_rhs_id: _input_rhs_id,
            output_id: _out_put_id,
        }
    }
}

/// Represents an entire arithmetic circuit, containing a collection of variables and gates.
#[derive(Serialize, Deserialize, Debug)]
pub struct ArithmeticCircuit {
    pub gate_count: u32,
    pub var_count: u32,
    pub vars: HashMap<u32, ArithmeticVar>,
    pub gates: HashMap<u32, ArithmeticNode>,
}

impl Default for ArithmeticCircuit {
    fn default() -> Self {
        Self::new()
    }
}

impl ArithmeticCircuit {
    pub fn new() -> ArithmeticCircuit {
        ArithmeticCircuit {
            gate_count: 0,
            var_count: 0,
            vars: HashMap::new(),
            gates: HashMap::new(),
        }
    }

    pub fn gate_count(&self) -> u32 {
        self.gate_count
    }

    pub fn var_count(&self) -> u32 {
        self.var_count
    }

    pub fn add_var(&mut self, var_id: u32, var_name: &str) -> &ArithmeticVar {
        println!(
            "[ArithmeticCircuit] Add var {} with id {}",
            var_name, var_id
        );

        // Not sure if var_count is needed
        self.var_count += 1;

        let var = ArithmeticVar::new(var_id, var_name.to_string());
        self.vars.insert(var_id, var);
        self.vars.get(&var_id).unwrap()
    }

    pub fn add_const_var(&mut self, var_id: u32, var_val: u32) -> &ArithmeticVar {
        println!(
            "[ArithmeticCircuit] var {} now has value {}",
            var_id, var_val
        );

        // Not sure if var_count is needed
        self.var_count += 1;

        let mut var = ArithmeticVar::new(var_id, var_val.to_string());
        var.is_const = true;
        var.const_value = var_val;
        self.vars.insert(var_id, var);
        self.vars.get(&var_id).unwrap()
    }

    pub fn get_var(&self, var_id: u32) -> &ArithmeticVar {
        self.vars.get(&var_id).unwrap()
    }

    pub fn get_var_mut(&mut self, var_id: u32) -> &mut ArithmeticVar {
        self.vars.get_mut(&var_id).unwrap()
    }

    //We support ADD, MUL, CADD, CMUL, DIV, CDIV, CINVERT, IFTHENELSE, FOR

    pub fn add_gate(
        &mut self,
        output_name: &str,
        output_id: u32,
        lhs_id: u32,
        rhs_id: u32,
        gate_type: AGateType,
    ) {
        self.gate_count += 1;
        self.add_var(output_id, output_name);
        let node = ArithmeticNode::new(self.gate_count, gate_type, lhs_id, rhs_id, output_id);
        let var_output = self.get_var(output_id);
        let var_lhs = self.get_var(lhs_id);
        let var_rhs = self.get_var(rhs_id);
        println!(
            "[ArithmeticCircuit] Gate added id {}: ({}, {}, {}) = ({}, {}, {}) {} ({}, {}, {})",
            node.gate_id,
            node.output_id,
            var_output.is_const,
            var_output.const_value,
            node.input_lhs_id,
            var_lhs.is_const,
            var_lhs.const_value,
            node.gate_type,
            node.input_rhs_id,
            var_rhs.is_const,
            var_rhs.const_value
        );
        self.gates.insert(self.gate_count, node);
    }

    pub fn replace_input_var_in_gate(&mut self, var_id: u32, new_var_id: u32) {
        for i in 1..(self.gate_count + 1) {
            if !self.gates.contains_key(&i) {
                continue;
            }
            let node = self.gates.get_mut(&(i)).unwrap();
            if node.input_lhs_id == var_id {
                node.input_lhs_id = new_var_id;
            }
            if node.input_rhs_id == var_id {
                node.input_rhs_id = new_var_id;
            }
        }
    }

    pub fn truncate_zero_add_gate(&mut self) {
        let mut zero_add_gate_indice = vec![];
        for i in 1..(self.gate_count + 1) {
            if !self.gates.contains_key(&i) {
                continue;
            }
            let node = self.gates.get(&(i)).unwrap();
            match node.gate_type {
                AGateType::AAdd => {
                    let var_output = self.get_var(node.output_id);
                    let var_lhs = self.get_var(node.input_lhs_id);
                    let var_rhs = self.get_var(node.input_rhs_id);
                    if var_lhs.is_const && var_lhs.const_value == 0 {
                        // var_output.var_id = var_rhs.var_id;
                        // var_output.var_name = var_rhs.var_name.to_string();
                        self.replace_input_var_in_gate(var_output.var_id, var_rhs.var_id);
                        zero_add_gate_indice.push(i);
                    } else if var_rhs.is_const && var_rhs.const_value == 0 {
                        // var_output.var_id = var_lhs.var_id;
                        // var_output.var_name = var_lhs.var_name;
                        self.replace_input_var_in_gate(var_output.var_id, var_lhs.var_id);
                        zero_add_gate_indice.push(i);
                    } else {
                        continue;
                    }
                }
                _ => {
                    continue;
                }
            }
        }
        for i in zero_add_gate_indice.iter() {
            self.gates.remove(i);
        }
    }

    pub fn print_ac(&self) {
        println!("[ArithmeticCircuit] Whole Arithmetic Circuit");
        for i in 1..(self.gate_count + 1) {
            if !self.gates.contains_key(&i) {
                continue;
            }
            let node = self.gates.get(&(i)).unwrap();
            // println!("[ArithmeticCircuit] Gate {}: {} = {} [{}] {}", i, anv.output_id, anv.input_lhs_id, anv.gate_type.to_string(), anv.input_rhs_id);
            let var_output = self.get_var(node.output_id);
            let var_lhs = self.get_var(node.input_lhs_id);
            let var_rhs = self.get_var(node.input_rhs_id);
            println!(
                "[ArithmeticCircuit] Gate id {}: ({}, {}, {}) = ({}, {}, {}) {} ({}, {}, {})",
                node.gate_id,
                node.output_id,
                var_output.is_const,
                var_output.const_value,
                node.input_lhs_id,
                var_lhs.is_const,
                var_lhs.const_value,
                node.gate_type,
                node.input_rhs_id,
                var_rhs.is_const,
                var_rhs.const_value
            );
        }
        // for (ank, anv) in self.gates.iter() {
        //     println!("Gate {}: {} = {} [{}] {}", ank, anv.output_id, anv.input_lhs_id, anv.gate_type.to_string(), anv.input_rhs_id);
        // }
    }

    pub fn serde(&self) {
        let serialized = serde_json::to_string(&self).unwrap();

        // Prints serialized = {"x":1,"y":2}
        println!("serialized = {}", serialized);

        // Convert the JSON string back to a Point.
        let deserialized: ArithmeticCircuit = serde_json::from_str(&serialized).unwrap();

        // Prints deserialized = Point { x: 1, y: 2 }
        println!("deserialized = {:#?}", deserialized);
    }
}

#[allow(dead_code)]
/// Represents a gate in its raw, unchecked form, used during parsing.
struct UncheckedGate {
    xref: usize,
    yref: Option<usize>,
    zref: usize,
    gate_type: GateType,
}

impl UncheckedGate {
    #[allow(dead_code)]
    pub fn parse(captures: Captures) -> Result<Self, ParseError> {
        let xref: usize = captures.name("xref").unwrap().as_str().parse()?;
        let yref: Option<usize> = captures
            .name("yref")
            .map(|yref| yref.as_str().parse())
            .transpose()?;
        let zref: usize = captures.name("zref").unwrap().as_str().parse()?;
        let gate_type = captures.name("gate").unwrap().as_str();

        let gate_type = match gate_type {
            "XOR" => GateType::Xor,
            "AND" => GateType::And,
            "INV" => GateType::Inv,
            _ => return Err(ParseError::UnsupportedGateType(gate_type.to_string())),
        };

        Ok(Self {
            xref,
            yref,
            zref,
            gate_type,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("uninitialized feed: {0}")]
    UninitializedFeed(usize),
    #[error("unsupported gate type: {0}")]
    UnsupportedGateType(String),
    #[error(transparent)]
    BuilderError(#[from] BuilderError),
}
