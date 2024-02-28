//! # Circuit Module
//!
//! This module defines the data structures used to represent the arithmetic circuit.

use crate::{program::ProgramError, runtime::generate_u32};
use circom_program_structure::ast::ExpressionInfixOpcode;
use log::debug;
use mpz_circuits::GateType;
use regex::Captures;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Represents a node in the circuit, with an identifier and a set of signals that it is connected to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    id: u32,
    signals: Vec<u32>,
    names: Vec<String>,
}

impl Node {
    /// Creates a new node.
    pub fn new(signal_id: u32, signal_name: String) -> Self {
        Self {
            id: generate_u32(),
            signals: vec![signal_id],
            names: vec![signal_name]
        }
    }

    /// Adds a set of signals to the node.
    pub fn add_signals(&mut self, signals: Vec<u32>, names: Vec<String>) {
        self.signals.extend(signals);
        self.names.extend(names);
    }

    /// Gets the signals of the node.
    pub fn get_signals(&self) -> Vec<u32> {
        self.signals.clone()
    }

    pub fn get_signals_names(&self) -> Vec<String> {
        self.names.clone()
    }

    /// Merges the signals of the node with another node, creating a new node.
    pub fn merge(&self, merge_node: &Node) -> Self {
        let mut new_node = Node {
            id: generate_u32(),
            signals: Vec::new(),
            names: Vec::new(),
        };

        new_node.add_signals(self.get_signals(), self.get_signals_names());
        new_node.add_signals(merge_node.get_signals(), merge_node.get_signals_names());

        new_node
    }

    /// Checks if the Node contains an output signal
    pub fn has_output(&self, circuit: &ArithmeticCircuit) -> bool {
        circuit.gates.iter().any(|gate| gate.output == self.id)
    }
}

/// Represents a circuit gate, with a left-hand input, right-hand input, and output node identifiers.
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

/// Represents an arithmetic circuit, with a set of variables and gates.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArithmeticCircuit {
    vars: HashMap<u32, Option<u32>>,
    nodes: Vec<Node>,
    gates: Vec<ArithmeticGate>,
}

impl ArithmeticCircuit {
    /// Creates a new arithmetic circuit.
    pub fn new() -> ArithmeticCircuit {
        ArithmeticCircuit {
            vars: HashMap::new(),
            nodes: Vec::new(),
            gates: Vec::new(),
        }
    }

    /// Adds a new signal variable to the circuit.
    pub fn add_signal(&mut self, id: u32, name: String) -> Result<(), CircuitError> {
        // Check that the variable isn't already declared
        if self.contains_var(&id) {
            return Err(CircuitError::CircuitVariableAlreadyDeclared);
        }
        self.vars.insert(id, None);

        // Create a new node for the signal
        let node = Node::new(id, name);
        debug!("New {:?}", node);

        self.nodes.push(node);
        Ok(())
    }

    /// Adds a new constant variable to the circuit.
    pub fn add_const(&mut self, value: u32) -> Result<(), CircuitError> {
        // Ignore if the constant is already declared
        if self.contains_var(&value) {
            return Ok(());
        }
        self.vars.insert(value, Some(value));

        // Create a new node for the constant
        let node = Node::new(value, format!("{}", value));
        debug!("New {:?}", node);

        self.nodes.push(node);
        Ok(())
    }

    /// Adds a new gate to the circuit.
    pub fn add_gate(
        &mut self,
        gate_type: AGateType,
        lhs_id: u32,
        rhs_id: u32,
        output_id: u32,
        lh_name: String,
        rh_name: String,
        o_name: String
    ) -> Result<(), CircuitError> {
        // Check that the inputs are declared
        if !self.contains_var(&lhs_id)
            || !self.contains_var(&rhs_id)
            || !self.contains_var(&output_id)
        {
            return Err(CircuitError::VariableNotDeclared);
        }

        match gate_type {
            AGateType::AAdd => {
                println!("{} = {} + {}", o_name, lh_name, rh_name);
            },
            AGateType::ADiv => todo!(),
            AGateType::AEq => todo!(),
            AGateType::AGEq => todo!(),
            AGateType::AGt => todo!(),
            AGateType::ALEq => todo!(),
            AGateType::ALt => todo!(),
            AGateType::AMul => {
                println!("{} = {} * {}", o_name, lh_name, rh_name);
            },
            AGateType::ANeq => todo!(),
            AGateType::ANone => todo!(),
            AGateType::ASub => {
                println!("{} = {} - {}", o_name, lh_name, rh_name);
            },
        };

        // Get the signal nodes
        let lhs_node = self.get_signal_node(lhs_id)?;
        let rhs_node = self.get_signal_node(rhs_id)?;
        let output_node = self.get_signal_node(output_id)?;

        // Create gate
        let gate = ArithmeticGate::new(
            self.gate_count(),
            gate_type,
            lhs_node.id,
            rhs_node.id,
            output_node.id,
        );
        debug!("New {:?} ", gate);

        self.gates.push(gate);
        Ok(())
    }

    /// Creates a connection between two signals in the circuit.
    /// This is done by finding the nodes that contain the signals and merging them.
    pub fn add_connection(&mut self, a: u32, b: u32, a_name: String, b_name: String) -> Result<(), CircuitError> {
        // Check that the endpoints are declared
        if !self.contains_var(&a) || !self.contains_var(&b) {
            return Err(CircuitError::VariableNotDeclared);
        }

        // Get the signal nodes
        let node_a = self.get_signal_node(a)?;
        let node_b = self.get_signal_node(b)?;

        // If both endpoints are in the same node, no action is needed
        if node_a.id == node_b.id {
            return Ok(());
        }

        // Check if both nodes are used as outputs in any gate
        if node_a.has_output(self) && node_b.has_output(self) {
            return Err(CircuitError::CannotMergeOutputNodes);
        }

        // Merge the nodes
        let merged_node = node_a.merge(&node_b);

        // Update connections in gates to point to the new merged node
        self.gates.iter_mut().for_each(|gate| {
            if gate.lh_input == node_a.id || gate.lh_input == node_b.id {
                gate.lh_input = merged_node.id;
            }
            if gate.rh_input == node_a.id || gate.rh_input == node_b.id {
                gate.rh_input = merged_node.id;
            }
            if gate.output == node_a.id || gate.output == node_b.id {
                gate.output = merged_node.id;
            }
        });

        // Remove the old nodes and add the new merged node
        self.nodes
            .retain(|node| node.id != node_a.id && node.id != node_b.id);
        self.nodes.push(merged_node);

        println!("{} = {}", a_name, b_name);

        Ok(())
    }

    /// Returns the node containing the given signal.
    fn get_signal_node(&self, signal_id: u32) -> Result<Node, CircuitError> {
        for node in &self.nodes {
            if node.signals.contains(&signal_id) {
                return Ok(node.clone());
            }
        }

        Err(CircuitError::NodeNotFound)
    }

    /// Checks if the variable exists
    pub fn contains_var(&self, var: &u32) -> bool {
        self.vars.contains_key(var)
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
    #[error("Cannot merge output nodes")]
    CannotMergeOutputNodes,
    #[error("Circuit variable already declared")]
    CircuitVariableAlreadyDeclared,
    #[error("Constant value already set for variable")]
    ConstantValueAlreadySet,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Node not found")]
    NodeNotFound,
    #[error("unsupported gate type: {0}")]
    UnsupportedGateType(String),
    #[error("Variable not declared")]
    VariableNotDeclared,
}

impl From<CircuitError> for ProgramError {
    fn from(e: CircuitError) -> Self {
        ProgramError::CircuitError(e)
    }
}
