//! # Circuit Module
//!
//! This module defines the data structures used to represent the arithmetic circuit.

use crate::program::ProgramError;
use bmr16_mpz::{
    arithmetic::{
        circuit::ArithmeticCircuit as MpzCircuit,
        ops::{add, cmul, mul, sub},
        types::CrtRepr,
        ArithCircuitError as MpzCircuitError,
    },
    ArithmeticCircuitBuilder,
};
use circom_program_structure::ast::ExpressionInfixOpcode;
use log::debug;
use serde::{Deserialize, Serialize};
use sim_circuit::circuit::{Circuit as SimCircuit, Gate as SimGate, Node as SimNode, Operation};
use std::collections::HashMap;
use thiserror::Error;
use sim_circuit::circuit::CircuitError as SimCircuitError;

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
    ASub,
    AXor,
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
            ExpressionInfixOpcode::BitXor => AGateType::AXor,
            _ => unimplemented!("Unsupported opcode"),
        }
    }
}

impl From<&AGateType> for Operation {
    fn from(gate: &AGateType) -> Self {
        match gate {
            AGateType::AAdd => Operation::Add,
            AGateType::ASub => Operation::Subtract,
            AGateType::AMul => Operation::Multiply,
            AGateType::ADiv => Operation::Divide,
            AGateType::AEq => Operation::Equals,
            AGateType::ANeq => Operation::NotEquals,
            AGateType::ALt => Operation::LessThan,
            AGateType::ALEq => Operation::LessOrEqual,
            AGateType::AGt => Operation::GreaterThan,
            AGateType::AGEq => Operation::GreaterOrEqual,
            AGateType::AXor => Operation::XorBitwise,
        }
    }
}

/// Represents a signal in the circuit, with a name and an optional value.
#[derive(Debug, Serialize, Deserialize)]
pub struct Signal {
    name: String,
    value: Option<u32>,
}

impl Signal {
    /// Creates a new signal.
    pub fn new(name: String, value: Option<u32>) -> Self {
        Self { name, value }
    }
}

/// Represents a node in the circuit, a collection of signals.
/// The `is_const` and `is_out` fields saves us some iterations over the signals and gates.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Node {
    is_const: bool,
    is_out: bool,
    signals: Vec<u32>,
}

impl Node {
    /// Creates a new empty node.
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
            is_const: false,
            is_out: false,
        }
    }

    /// Creates a new node with an initial signal.
    pub fn new_with_signal(signal_id: u32, is_const: bool, is_out: bool) -> Self {
        Self {
            signals: vec![signal_id],
            is_const,
            is_out,
        }
    }

    /// Adds a set of signals to the node.
    pub fn add_signals(&mut self, signals: &Vec<u32>) {
        self.signals.extend(signals);
    }

    /// Checks if the node contains a signal.
    pub fn contains_signal(&self, signal_id: &u32) -> bool {
        self.signals.contains(signal_id)
    }

    /// Gets the signals of the node.
    pub fn get_signals(&self) -> &Vec<u32> {
        &self.signals
    }

    /// Sets the node as an output node.
    pub fn set_output(&mut self, is_out: bool) {
        self.is_out = is_out;
    }

    /// Sets the node as a constant node.
    pub fn set_const(&mut self, is_const: bool) {
        self.is_const = is_const;
    }
}

/// Represents a circuit gate, with a left-hand input, right-hand input, and output node identifiers.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArithmeticGate {
    op: AGateType,
    lh_in: u32,
    rh_in: u32,
    out: u32,
}

impl ArithmeticGate {
    /// Creates a new gate.
    pub fn new(op: AGateType, lh_in: u32, rh_in: u32, out: u32) -> Self {
        Self {
            op,
            lh_in,
            rh_in,
            out,
        }
    }
}

/// Represents an arithmetic circuit, with a set of variables and gates.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ArithmeticCircuit {
    node_count: u32,
    signals: HashMap<u32, Signal>,
    nodes: HashMap<u32, Node>,
    gates: Vec<ArithmeticGate>,
}

impl ArithmeticCircuit {
    /// Creates a new arithmetic circuit.
    pub fn new() -> ArithmeticCircuit {
        ArithmeticCircuit {
            node_count: 0,
            signals: HashMap::new(),
            nodes: HashMap::new(),
            gates: Vec::new(),
        }
    }

    /// Adds a new signal to the circuit.
    pub fn add_signal(
        &mut self,
        id: u32,
        name: String,
        value: Option<u32>,
    ) -> Result<(), CircuitError> {
        // Check that the signal isn't already declared
        if self.signals.contains_key(&id) {
            return Err(CircuitError::SignalAlreadyDeclared);
        }

        // Create a new signal
        let signal = Signal::new(name, value);
        self.signals.insert(id, signal);

        // Create a new node
        let node = Node::new_with_signal(id, false, value.is_some());
        debug!("{:?}", node);
        let node_id = self.get_node_id();
        self.nodes.insert(node_id, node);

        Ok(())
    }

    /// Adds a new gate to the circuit.
    pub fn add_gate(
        &mut self,
        gate_type: AGateType,
        lhs_signal_id: u32,
        rhs_signal_id: u32,
        output_signal_id: u32,
    ) -> Result<(), CircuitError> {
        // Get the signal node ids
        let node_ids = {
            let mut nodes: [u32; 3] = [0; 3];

            for (&id, node) in self.nodes.iter() {
                if node.contains_signal(&lhs_signal_id) {
                    nodes[0] = id;
                }
                if node.contains_signal(&rhs_signal_id) {
                    nodes[1] = id;
                }
                if node.contains_signal(&output_signal_id) {
                    nodes[2] = id;
                }
            }

            nodes
        };

        // Set the output node as an output node
        self.nodes.get_mut(&node_ids[2]).unwrap().set_output(true);

        // Create gate
        let gate = ArithmeticGate::new(gate_type, node_ids[0], node_ids[1], node_ids[2]);
        debug!("{:?}", gate);
        self.gates.push(gate);

        Ok(())
    }

    /// Creates a connection between two signals in the circuit.
    /// This is finding the nodes that contain these signals and merging them.
    pub fn add_connection(&mut self, a: u32, b: u32) -> Result<(), CircuitError> {
        // Get the signal node ids
        let n = Node::new();
        let nodes = {
            let mut nodes: [(u32, &Node); 2] = [(0, &n); 2];

            for (&id, node) in self.nodes.iter() {
                if node.contains_signal(&a) {
                    nodes[0] = (id, node);
                }
                if node.contains_signal(&b) {
                    nodes[1] = (id, node);
                }
            }

            nodes
        };

        let (node_a_id, node_a) = nodes[0];
        let (node_b_id, node_b) = nodes[1];

        // If both signals are in the same node, no action is needed
        if node_a_id == node_b_id {
            return Ok(());
        }
        // Check for output and constant nodes
        if node_a.is_out && node_b.is_out {
            return Err(CircuitError::CannotMergeOutputNodes);
        }

        if node_a.is_const && node_b.is_const {
            return Err(CircuitError::CannotMergeConstantNodes);
        }

        // Merge the nodes into a new node
        let mut merged_node = Node::new();

        // Set the new node as an output and constant
        merged_node.set_output(node_a.is_out || node_b.is_out);
        merged_node.set_const(node_a.is_const || node_b.is_const);

        merged_node.add_signals(node_a.get_signals());
        merged_node.add_signals(node_b.get_signals());

        let merged_node_id = self.get_node_id();

        // Update connections in gates to point to the new merged node
        self.gates.iter_mut().for_each(|gate| {
            if gate.lh_in == node_a_id || gate.lh_in == node_b_id {
                gate.lh_in = merged_node_id;
            }
            if gate.rh_in == node_a_id || gate.rh_in == node_b_id {
                gate.rh_in = merged_node_id;
            }
            if gate.out == node_a_id || gate.out == node_b_id {
                gate.out = merged_node_id;
            }
        });

        // Remove the old nodes and insert the new merged node
        self.nodes.remove(&node_a_id);
        self.nodes.remove(&node_b_id);
        self.nodes.insert(merged_node_id, merged_node);

        Ok(())
    }

    /// Generates a circuit report with input and output signals information.
    pub fn generate_circuit_report(&self) -> Result<CircuitReport, CircuitError> {
        // Split input and output nodes
        let mut input_nodes = Vec::new();
        let mut output_nodes = Vec::new();
        self.nodes.iter().for_each(|(&id, node)| {
            if node.is_out {
                output_nodes.push(id);
            } else {
                input_nodes.push(id);
            }
        });

        // Remove output nodes that are inputs to gates
        output_nodes.retain(|&id| {
            self.gates
                .iter()
                .all(|gate| gate.lh_in != id && gate.rh_in != id)
        });

        // Sort
        input_nodes.sort_unstable();
        output_nodes.sort_unstable();

        // Generate reports
        let inputs = self.generate_signal_reports(&input_nodes);
        let outputs = self.generate_signal_reports(&output_nodes);

        Ok(CircuitReport { inputs, outputs })
    }

    /// Builds an arithmetic circuit using the mpz circuit builder.
    pub fn build_mpz_circuit(&self, report: &CircuitReport) -> Result<MpzCircuit, CircuitError> {
        let builder = ArithmeticCircuitBuilder::new();

        // Initialize CRT signals map with the circuit inputs
        let mut crt_signals: HashMap<u32, CrtRepr> =
            report
                .inputs
                .iter()
                .try_fold(HashMap::new(), |mut acc, signal| {
                    let input = builder
                        .add_input::<u32>(signal.names[0].to_string())
                        .map_err(CircuitError::MPZCircuitError)?;
                    acc.insert(signal.id, input.repr);
                    Ok::<_, CircuitError>(acc)
                })?;

        // Initialize a vec for indices of gates that need processing
        let mut to_process = std::collections::VecDeque::new();
        to_process.extend(0..self.gates.len());

        while let Some(index) = to_process.pop_front() {
            let gate = &self.gates[index];

            if let (Some(lh_in_repr), Some(rh_in_repr)) =
                (crt_signals.get(&gate.lh_in), crt_signals.get(&gate.rh_in))
            {
                let result_repr = match gate.op {
                    AGateType::AAdd => {
                        add(&mut builder.state().borrow_mut(), lh_in_repr, rh_in_repr)
                            .map_err(|e| e.into())
                    }
                    AGateType::AMul => {
                        // Get the constant value from one of the signals if available
                        let constant_value = self
                            .signals
                            .get(&gate.lh_in)
                            .and_then(|signal| signal.value.map(|v| v as u64))
                            .or_else(|| {
                                self.signals
                                    .get(&gate.rh_in)
                                    .and_then(|signal| signal.value.map(|v| v as u64))
                            });

                        // Perform multiplication depending on whether one input is a constant
                        if let Some(value) = constant_value {
                            Ok::<_, CircuitError>(cmul(
                                &mut builder.state().borrow_mut(),
                                lh_in_repr,
                                value,
                            ))
                        } else {
                            mul(&mut builder.state().borrow_mut(), lh_in_repr, rh_in_repr)
                                .map_err(|e| e.into())
                        }
                    }
                    AGateType::ASub => {
                        sub(&mut builder.state().borrow_mut(), lh_in_repr, rh_in_repr)
                            .map_err(|e| e.into())
                    }
                    _ => {
                        return Err(CircuitError::UnsupportedGateType(format!(
                            "{:?} not supported by MPZ",
                            gate.op
                        )))
                    }
                }?;

                crt_signals.insert(gate.out, result_repr);
            } else {
                // Not ready to process, push back for later attempt.
                to_process.push_back(index);
            }
        }

        // Add output signals
        for signal in &report.outputs {
            let output_repr = crt_signals
                .get(&signal.id)
                .ok_or_else(|| CircuitError::UnprocessedNode)?;
            builder.add_output(output_repr);
        }

        builder
            .build()
            .map_err(|_| CircuitError::MPZCircuitBuilderError)
    }

    /// Builds a sim circuit instance.
    pub fn build_sim_circuit(&self) -> Result<SimCircuit, CircuitError> {
        let mut sim_circuit = SimCircuit::new();

        // Add nodes
        for (&id, node) in &self.nodes {
            let mut new_node = SimNode::new();
            if let Some(value) = node
                .signals
                .first()
                .and_then(|&sig_id| self.signals.get(&sig_id).and_then(|sig| sig.value))
            {
                new_node.set_value(value);
            }
            sim_circuit.add_node(id, new_node)?;
        }

        // Add gates
        for gate in &self.gates {
            let operation = Operation::from(&gate.op);
            let sim_gate = SimGate::new(operation, gate.lh_in, gate.rh_in, gate.out);
            sim_circuit.add_gate(sim_gate)?;
        }

        Ok(sim_circuit)
    }

    /// Returns a node id and increments the count.
    fn get_node_id(&mut self) -> u32 {
        self.node_count += 1;
        self.node_count
    }

    /// Generates signal reports for a set of node IDs.
    fn generate_signal_reports(&self, nodes: &[u32]) -> Vec<SignalReport> {
        nodes
            .iter()
            .map(|&id| {
                let signals = self
                    .nodes
                    .get(&id)
                    .expect("Node ID not found in node map")
                    .get_signals();

                let (names, value) = signals.iter().fold((Vec::new(), None), |mut acc, &sig_id| {
                    let signal = self
                        .signals
                        .get(&sig_id)
                        .expect("Signal ID not found in signal map");

                    if !signal.name.contains("random_") {
                        acc.0.push(signal.name.clone());
                    }
                    if signal.value.is_some() {
                        acc.1 = signal.value;
                    }
                    acc
                });

                SignalReport { id, names, value }
            })
            .collect()
    }
}

/// The full circuit report, containing input and output signals information.
#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitReport {
    inputs: Vec<SignalReport>,
    outputs: Vec<SignalReport>,
}

/// A single node report, with a list of signal names and an optional value.
#[derive(Debug, Serialize, Deserialize)]
pub struct SignalReport {
    id: u32,
    names: Vec<String>,
    value: Option<u32>,
}

#[derive(Debug, Error)]
pub enum CircuitError {
    #[error("Cannot merge constant nodes")]
    CannotMergeConstantNodes,
    #[error("Cannot merge output nodes")]
    CannotMergeOutputNodes,
    #[error("Constant value already set for variable")]
    ConstantValueAlreadySet,
    #[error("Signal is not connected to any node")]
    DisconnectedSignal,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("MPZ arithmetic circuit error: {0}")]
    MPZCircuitError(MpzCircuitError),
    #[error("MPZ arithmetic circuit builder error")]
    MPZCircuitBuilderError,
    #[error("Circuit simulation error")]
    SimCircuitError(SimCircuitError),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Signal already declared")]
    SignalAlreadyDeclared,
    #[error("unsupported gate type: {0}")]
    UnsupportedGateType(String),
    #[error("Unprocessed node")]
    UnprocessedNode,
}

impl From<CircuitError> for ProgramError {
    fn from(e: CircuitError) -> Self {
        ProgramError::CircuitError(e)
    }
}

impl From<MpzCircuitError> for CircuitError {
    fn from(e: MpzCircuitError) -> Self {
        CircuitError::MPZCircuitError(e)
    }
}

impl From<SimCircuitError> for CircuitError {
    fn from(e: SimCircuitError) -> Self {
        CircuitError::SimCircuitError(e)
    }
}
