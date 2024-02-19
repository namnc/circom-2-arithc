//! # Circuit Module
//!
//! This module defines the data structures used to represent the arithmetic circuit.

use circom_program_structure::ast::ExpressionInfixOpcode;
use log::debug;
use mpz_circuits::GateType;
use regex::Captures;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};
use thiserror::Error;

/// Types of gates that can be used in an arithmetic circuit.
#[derive(Debug, Serialize, Deserialize)]
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

/// Represents a variable in an arithmetic circuit, with an optional constant value.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArithmeticVar {
    id: u32,
    value: Option<u32>,
}

impl ArithmeticVar {
    /// Creates a new arithmetic variable.
    pub fn new(id: u32, value: Option<u32>) -> Self {
        Self { id, value }
    }

    /// Returns whether the variable is a constant.
    pub fn is_const(&self) -> bool {
        self.value.is_some()
    }

    /// Sets the value of the variable, if it is not already set.
    pub fn set_value(&mut self, value: u32) -> Result<(), CircuitError> {
        if self.value.is_some() {
            return Err(CircuitError::ConstantValueAlreadySet);
        }

        self.value = Some(value);

        Ok(())
    }
}

/// Represents a circuit gate, with a left-hand input, right-hand input, and output identifiers.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArithmeticGate {
    id: u32,
    gate_type: AGateType,
    lh_input: u32,
    rh_input: u32,
    output: u32,
}

impl ArithmeticGate {
    /// Creates a new gate.
    pub fn new(id: u32, gate_type: AGateType, lh_input: u32, rh_input: u32, output: u32) -> Self {
        Self {
            id,
            gate_type,
            lh_input,
            rh_input,
            output,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    from: u32,
    to: u32,
}

/// Represents an arithmetic circuit, with a set of variables and gates.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArithmeticCircuit {
    vars: HashMap<u32, ArithmeticVar>,
    connections: Vec<Connection>,
    gates: Vec<ArithmeticGate>,
}

impl Default for ArithmeticCircuit {
    fn default() -> Self {
        Self::new()
    }
}

impl ArithmeticCircuit {
    /// Creates a new arithmetic circuit.
    pub fn new() -> ArithmeticCircuit {
        ArithmeticCircuit {
            vars: HashMap::new(),
            connections: Vec::new(),
            gates: Vec::new(),
        }
    }

    /// Adds a new signal variable to the circuit.
    pub fn add_signal(&mut self, id: u32) -> Result<(), CircuitError> {
        // Check that the variable isn't already declared
        if self.vars.contains_key(&id) {
            return Err(CircuitError::CircuitVariableAlreadyDeclared);
        }

        debug!("New signal {}", id);

        self.vars.insert(id, ArithmeticVar::new(id, None));

        Ok(())
    }

    /// Adds a new constant variable to the circuit.
    pub fn add_const(&mut self, value: u32) -> Result<(), CircuitError> {
        // We're using the value as the identifier for the constant
        // Check that the constant isn't already declared
        if self.vars.contains_key(&value) {
            return Err(CircuitError::CircuitVariableAlreadyDeclared);
        }

        debug!("New constant {}", value);

        self.vars
            .insert(value, ArithmeticVar::new(value, Some(value)));

        Ok(())
    }

    /// Returns the variable with the given identifier.
    pub fn get_var(&self, id: u32) -> Result<&ArithmeticVar, CircuitError> {
        match self.vars.get(&id) {
            Some(var) => Ok(var),
            None => Err(CircuitError::CircuitVariableAlreadyDeclared),
        }
    }

    /// Returns the variable with the given identifier, mutably.
    pub fn get_var_mut(&mut self, id: u32) -> Result<&mut ArithmeticVar, CircuitError> {
        match self.vars.get_mut(&id) {
            Some(var) => Ok(var),
            None => Err(CircuitError::CircuitVariableAlreadyDeclared),
        }
    }

    /// Adds a new gate to the circuit.
    pub fn add_gate(
        &mut self,
        gate_type: AGateType,
        lhs_id: u32,
        rhs_id: u32,
        output_id: u32,
    ) -> Result<(), CircuitError> {
        // Add the output variable
        self.add_signal(output_id)?;

        // Check that the inputs are declared
        if !self.vars.contains_key(&lhs_id) || !self.vars.contains_key(&rhs_id) {
            return Err(CircuitError::VariableNotDeclared);
        }

        // Create gate
        let gate = ArithmeticGate::new(self.gate_count(), gate_type, lhs_id, rhs_id, output_id);
        debug!("New gate {:?} ", gate);

        self.gates.push(gate);

        Ok(())
    }

    /// Adds a new connection to the circuit.
    pub fn add_connection(&mut self, from: u32, to: u32) -> Result<(), CircuitError> {
        // Check for direct or reverse duplicate connections
        if from == to
            || self
                .connections
                .iter()
                .any(|c| (c.from == from && c.to == to) || (c.from == to && c.to == from))
        {
            return Ok(());
        }

        debug!("New connection from {} to {}", from, to);

        self.connections.push(Connection { from, to });

        Ok(())
    }

    /// Returns the number of gates in the circuit.
    pub fn gate_count(&self) -> u32 {
        self.gates.len() as u32
    }
}

#[allow(dead_code)]
/// Represents a gate in its raw, unchecked form, used during parsing.
pub struct UncheckedGate {
    xref: usize,
    yref: Option<usize>,
    zref: usize,
    gate_type: GateType,
}

impl UncheckedGate {
    #[allow(dead_code)]
    pub fn parse(captures: Captures) -> Result<Self, CircuitError> {
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
            _ => return Err(CircuitError::UnsupportedGateType(gate_type.to_string())),
        };

        Ok(Self {
            xref,
            yref,
            zref,
            gate_type,
        })
    }
}

#[derive(Debug, Error)]
pub enum CircuitError {
    #[error("Circuit variable already declared")]
    CircuitVariableAlreadyDeclared,
    #[error("Constant value already set for variable")]
    ConstantValueAlreadySet,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("unsupported gate type: {0}")]
    UnsupportedGateType(String),
    #[error("Variable not declared")]
    VariableNotDeclared,
}
