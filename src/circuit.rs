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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    id: u32,
    name: String,
    value: Option<u32>,
}

impl Signal {
    pub fn new(id: u32, name: String, value: Option<u32>) -> Self {
        Self { id, name, value }
    }

    pub fn is_const(&self) -> bool {
        self.value.is_some()
    }
}

/// Represents a node in the circuit, with an identifier and a set of signals that it is connected to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    id: u32,
    signals: Vec<Signal>,
}

impl Node {
    /// Creates a new node.
    pub fn new(signal: Signal) -> Self {
        Self {
            id: generate_u32(),
            signals: vec![signal],
        }
    }

    /// Adds a set of signals to the node.
    pub fn add_signals(&mut self, signals: Vec<Signal>) {
        self.signals.extend(signals);
    }

    /// Gets the signals of the node.
    pub fn get_signals(&self) -> Vec<Signal> {
        self.signals.clone()
    }

    /// Merges the signals of the node with another node, creating a new node.
    pub fn merge(&self, merge_node: &Node) -> Self {
        let mut new_node = Node {
            id: generate_u32(),
            signals: Vec::new(),
        };

        new_node.add_signals(self.get_signals());
        new_node.add_signals(merge_node.get_signals());

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
    signals: HashMap<u32, Signal>,
    nodes: Vec<Node>,
    gates: Vec<ArithmeticGate>,
}

impl ArithmeticCircuit {
    /// Creates a new arithmetic circuit.
    pub fn new() -> ArithmeticCircuit {
        ArithmeticCircuit {
            signals: HashMap::new(),
            nodes: Vec::new(),
            gates: Vec::new(),
        }
    }

    /// Adds a new signal variable to the circuit.
    pub fn add_signal(&mut self, id: u32, name: String) -> Result<(), CircuitError> {
        // Check that the variable isn't already declared
        if self.is_signal_declared(&id) {
            return Err(CircuitError::CircuitVariableAlreadyDeclared);
        }

        // Create a new Signal
        let signal = Signal::new(id, name, None);

        // Store the signal data
        self.signals.insert(signal.id, signal.clone());

        // Create a new node for the signal
        let node = Node::new(signal);
        debug!("New {:?}", node);

        self.nodes.push(node);
        Ok(())
    }

    /// Adds a new constant variable to the circuit.
    pub fn add_const(&mut self, value: u32, name: String) -> Result<(), CircuitError> {
        // Ignore if the constant is already declared
        if self.is_signal_declared(&value) {
            return Ok(());
        }

        // Create a new constant Signal
        let signal = Signal::new(value, name, Some(value));

        // Store the signal data
        self.signals.insert(signal.id, signal.clone());

        // Create a new node for the constant
        let node = Node::new(signal);
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
    ) -> Result<(), CircuitError> {
        // Check that the inputs are declared
        if !self.is_signal_declared(&lhs_id)
            || !self.is_signal_declared(&rhs_id)
            || !self.is_signal_declared(&output_id)
        {
            return Err(CircuitError::VariableNotDeclared);
        }

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
    pub fn add_connection(&mut self, a: u32, b: u32) -> Result<(), CircuitError> {
        // Check that the endpoints are declared
        if !self.is_signal_declared(&a) || !self.is_signal_declared(&b) {
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

        Ok(())
    }

    /// Returns the node containing the given signal.
    fn get_signal_node(&self, signal_id: u32) -> Result<Node, CircuitError> {
        for node in &self.nodes {
            for signal in &node.signals {
                if signal.id == signal_id {
                    return Ok(node.clone());
                }
            }
        }

        Err(CircuitError::NodeNotFound)
    }

    /// Checks if the signal exists
    pub fn is_signal_declared(&self, id: &u32) -> bool {
        self.signals.contains_key(id)
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
