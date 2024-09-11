//! # Circuit Module
//!
//! This module defines the data structures used to represent the arithmetic circuit.

use crate::{
    a_gate_type::AGateType, cli::ValueType, program::ProgramError,
    topological_sort::topological_sort,
};
use bristol_circuit::{BristolCircuit, CircuitInfo, ConstantInfo, Gate};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArithmeticGate {
    pub op: AGateType,
    pub lh_in: u32,
    pub rh_in: u32,
    pub out: u32,
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

/// Compilation data structure representing an arithmetic circuit with extra information, including
/// a set of variables and gates.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Compiler {
    node_count: u32,
    inputs: HashMap<u32, String>,
    outputs: HashMap<u32, String>,
    signals: HashMap<u32, Signal>,
    nodes: HashMap<u32, Node>,
    gates: Vec<ArithmeticGate>,
    value_type: ValueType,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            node_count: 0,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            signals: HashMap::new(),
            nodes: HashMap::new(),
            gates: Vec::new(),
            value_type: Default::default(),
        }
    }

    pub fn add_inputs(&mut self, inputs: HashMap<u32, String>) {
        self.inputs.extend(inputs);
    }

    pub fn add_outputs(&mut self, outputs: HashMap<u32, String>) {
        self.outputs.extend(outputs);
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
        let node = Node::new_with_signal(id, value.is_some(), false);
        debug!("{:?}", node);
        let node_id = self.get_node_id();
        self.nodes.insert(node_id, node);

        Ok(())
    }

    pub fn get_signals(&self, filter: String) -> HashMap<u32, String> {
        let mut ret = HashMap::new();
        for (signal_id, signal) in self.signals.iter() {
            if signal.name.starts_with(filter.as_str()) {
                ret.insert(*signal_id, signal.name.to_string());
            }
        }
        ret
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

    pub fn update_type(&mut self, value_type: ValueType) -> Result<(), CircuitError> {
        self.value_type = value_type;

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

        Ok(CircuitReport {
            inputs,
            outputs,
            value_type: self.value_type,
        })
    }

    pub fn build_circuit(&self) -> Result<BristolCircuit, CircuitError> {
        // First build up these maps so we can easily see which node id to use
        let mut input_to_node_id = HashMap::<String, u32>::new();
        let mut constant_to_node_id_and_value = HashMap::<String, (u32, String)>::new();
        let mut output_to_node_id = HashMap::<String, u32>::new();

        for (node_id, node) in self.nodes.iter() {
            // Each node has a list of signal ids which all correspond to that node
            // The compiler associates IO with signals, so here we bridge the gap so we get
            // IO <=> node instead of IO <=> signal <=> node
            for signal_id in node.get_signals() {
                if let Some(input_name) = self.inputs.get(signal_id) {
                    let prev = input_to_node_id.insert(input_name.clone(), *node_id);

                    if prev.is_some() {
                        return Err(CircuitError::Inconsistency {
                            message: format!("Duplicate input {}", input_name),
                        });
                    }
                }

                if let Some(output_name) = self.outputs.get(signal_id) {
                    let prev = output_to_node_id.insert(output_name.clone(), *node_id);

                    if prev.is_some() {
                        return Err(CircuitError::Inconsistency {
                            message: format!("Duplicate output {}", output_name),
                        });
                    }
                }

                let signal = &self.signals[signal_id];

                if let Some(value) = signal.value {
                    constant_to_node_id_and_value.insert(
                        format!("{}_{}", signal.name.clone(), signal_id),
                        (*node_id, value.to_string()),
                    );
                }
            }
        }

        {
            // We want inputs at the start and outputs at the end
            // We won't be able to do that if a node is used for both input and output
            // That shouldn't happen, so we check here that it doesn't happen

            let node_id_to_input_name = input_to_node_id
                .iter()
                .map(|(name, node_id)| (node_id, name))
                .collect::<HashMap<_, _>>();

            for (output_name, output_node_id) in &output_to_node_id {
                if let Some(input_name) = node_id_to_input_name.get(output_node_id) {
                    return Err(CircuitError::Inconsistency {
                        message: format!(
                            "Node {} used for both input {} and output {}",
                            output_node_id, input_name, output_name
                        ),
                    });
                }
            }
        }

        // Now node ids are like wire ids, but the compiler generates them in a way that leaves a
        // lot of gaps. So we assign new wire ids so they'll be sequential instead. We also do this
        // ensure inputs are at the start and outputs are at the end.
        let mut node_id_to_wire_id = HashMap::<u32, u32>::new();
        let mut next_wire_id = 0;

        // First inputs
        for node_id in input_to_node_id.values() {
            node_id_to_wire_id.insert(*node_id, next_wire_id);
            next_wire_id += 1;
        }

        // For the intermediate nodes, we need the gates in topological order so that the wires are
        // assigned in the order they are needed. The topological order is also needed to comply
        // with bristol format and allow for easy evaluation.

        let mut node_id_to_required_gate = HashMap::<u32, usize>::new();

        for (gate_id, gate) in self.gates.iter().enumerate() {
            // the gate.out node depends on this gate
            node_id_to_required_gate.insert(gate.out, gate_id);
        }

        let sorted_gate_ids = topological_sort(self.gates.len(), &|gate_id: usize| {
            let gate = &self.gates[gate_id];
            let mut deps = Vec::<usize>::new();

            if let Some(required_gate_id) = node_id_to_required_gate.get(&gate.lh_in) {
                deps.push(*required_gate_id);
            }

            if let Some(required_gate_id) = node_id_to_required_gate.get(&gate.rh_in) {
                deps.push(*required_gate_id);
            }

            deps
        })?;

        let output_node_ids = output_to_node_id.values().collect::<HashSet<_>>();

        // Now that the gates are in order, we can assign wire ids to each node in the order they
        // are seen
        for gate_id in &sorted_gate_ids {
            let gate = &self.gates[*gate_id];

            for node_id in &[gate.lh_in, gate.rh_in, gate.out] {
                if output_node_ids.contains(node_id) {
                    // Output wires are excluded so that they can all be at the end
                    continue;
                }

                if node_id_to_wire_id.contains_key(node_id) {
                    continue;
                }

                node_id_to_wire_id.insert(*node_id, next_wire_id);
                next_wire_id += 1;
            }
        }

        // Assign wire ids to output nodes
        for node_id in output_to_node_id.values() {
            node_id_to_wire_id.insert(*node_id, next_wire_id);
            next_wire_id += 1;
        }

        // Now we can create the new gates using topological order and the new wire ids
        let mut new_gates = Vec::<Gate>::new();
        for gate_id in sorted_gate_ids {
            let gate = &self.gates[gate_id];

            new_gates.push(Gate {
                inputs: vec![
                    node_id_to_wire_id[&gate.lh_in] as usize,
                    node_id_to_wire_id[&gate.rh_in] as usize,
                ],
                outputs: vec![node_id_to_wire_id[&gate.out] as usize],
                op: gate.op.to_string(),
            });
        }

        let mut constants = HashMap::<String, ConstantInfo>::new();

        for (name, (node_id, value)) in constant_to_node_id_and_value {
            constants.insert(
                name,
                ConstantInfo {
                    value,
                    wire_index: node_id_to_wire_id[&node_id] as usize,
                },
            );
        }

        Ok(BristolCircuit {
            wire_count: next_wire_id as usize,
            info: CircuitInfo {
                input_name_to_wire_index: input_to_node_id
                    .iter()
                    .map(|(name, node_id)| (name.clone(), node_id_to_wire_id[node_id] as usize))
                    .collect(),
                constants,
                output_name_to_wire_index: output_to_node_id
                    .iter()
                    .map(|(name, node_id)| (name.clone(), node_id_to_wire_id[node_id] as usize))
                    .collect(),
            },
            gates: new_gates,
            io_widths: None,
        })
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
    value_type: ValueType,
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
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Signal already declared")]
    SignalAlreadyDeclared,
    #[error("unsupported gate type: {0}")]
    UnsupportedGateType(String),
    #[error("Unprocessed node")]
    UnprocessedNode,
    #[error("Cyclic dependency: {message}")]
    CyclicDependency { message: String },
    #[error("Inconsistency: {message}")]
    Inconsistency { message: String },
    #[error("Parsing error: {message}")]
    ParsingError { message: String },
}

impl From<CircuitError> for ProgramError {
    fn from(e: CircuitError) -> Self {
        ProgramError::CircuitError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_with_signal() {
        let node = Node::new_with_signal(1, true, false);
        assert_eq!(node.signals.len(), 1);
        assert_eq!(node.signals[0], 1);
        assert!(node.is_const);
        assert!(!node.is_out);
    }

    #[test]
    fn test_node_add_signal() {
        let mut node = Node::new();
        node.add_signals(&vec![1, 2, 3]);
        assert_eq!(node.signals.len(), 3);
        assert!(node.contains_signal(&1));
        assert!(node.contains_signal(&2));
        assert!(node.contains_signal(&3));
    }

    #[test]
    fn test_node_contains_signal() {
        let node = Node::new_with_signal(1, true, false);
        assert!(node.contains_signal(&1));
        assert!(!node.contains_signal(&2));
    }

    #[test]
    fn test_node_set_output() {
        let mut node = Node::new();
        node.set_output(true);
        assert!(node.is_out);
    }

    #[test]
    fn test_node_set_const() {
        let mut node = Node::new();
        node.set_const(true);
        assert!(node.is_const);
    }

    #[test]
    fn test_compiler_add_inputs() {
        let mut compiler = Compiler::new();
        let mut inputs = HashMap::new();
        inputs.insert(1, String::from("input1"));
        inputs.insert(2, String::from("input2"));
        compiler.add_inputs(inputs);

        assert_eq!(compiler.inputs.len(), 2);
        assert_eq!(compiler.inputs[&1], "input1");
        assert_eq!(compiler.inputs[&2], "input2");
    }

    #[test]
    fn test_compiler_add_outputs() {
        let mut compiler = Compiler::new();
        let mut outputs = HashMap::new();
        outputs.insert(3, String::from("output1"));
        outputs.insert(4, String::from("output2"));
        compiler.add_outputs(outputs);

        assert_eq!(compiler.outputs.len(), 2);
        assert_eq!(compiler.outputs[&3], "output1");
        assert_eq!(compiler.outputs[&4], "output2");
    }

    #[test]
    fn test_compiler_add_signal() {
        let mut compiler = Compiler::new();
        let result = compiler.add_signal(1, String::from("signal1"), None);

        assert!(result.is_ok());
        assert_eq!(compiler.signals.len(), 1);
        assert!(compiler.signals.contains_key(&1));
        assert_eq!(compiler.signals[&1].name, "signal1");
    }

    #[test]
    fn test_compiler_add_duplicated_signal() {
        let mut compiler = Compiler::new();
        compiler
            .add_signal(1, String::from("signal1"), None)
            .unwrap();
        let result = compiler.add_signal(1, String::from("signal1"), None);

        assert!(matches!(result, Err(CircuitError::SignalAlreadyDeclared)));
    }

    #[test]
    fn test_compiler_get_signals() {
        let mut compiler = Compiler::new();
        compiler
            .add_signal(1, String::from("signal1"), None)
            .unwrap();
        compiler
            .add_signal(2, String::from("filter_signal"), None)
            .unwrap();
        let filtered_signals = compiler.get_signals(String::from("filter"));

        assert_eq!(filtered_signals.len(), 1);
        assert_eq!(filtered_signals[&2], "filter_signal");
    }

    #[test]
    fn test_compiler_add_gate() {
        let mut compiler = Compiler::new();
        compiler
            .add_signal(1, String::from("signal1"), None)
            .unwrap();
        compiler
            .add_signal(2, String::from("signal2"), None)
            .unwrap();
        compiler
            .add_signal(3, String::from("signal3"), None)
            .unwrap();

        let result = compiler.add_gate(AGateType::AAdd, 1, 2, 3);

        assert!(result.is_ok());
        assert_eq!(compiler.gates.len(), 1);
        let gate = &compiler.gates[0];
        assert_eq!(gate.op, AGateType::AAdd);
        assert_eq!(gate.lh_in, 1);
        assert_eq!(gate.rh_in, 2);
        assert_eq!(gate.out, 3);
    }

    #[test]
    fn test_compiler_add_connection() {
        let mut compiler = Compiler::new();
        compiler
            .add_signal(1, String::from("signal1"), None)
            .unwrap();
        compiler
            .add_signal(2, String::from("signal2"), None)
            .unwrap();
        compiler
            .add_signal(3, String::from("signal3"), None)
            .unwrap();

        // Adding connection between signals 1 and 2
        let result = compiler.add_connection(1, 2);

        assert!(result.is_ok());
        assert_eq!(compiler.nodes.len(), 2);

        // Assert new node contains both signals
        let node = compiler.nodes.get(&4).unwrap();
        assert_eq!(node.signals.len(), 2);
        assert!(node.contains_signal(&1));
        assert!(node.contains_signal(&2));
    }

    #[test]
    fn test_compiler_add_connection_same_node() {
        let mut compiler = Compiler::new();
        compiler
            .add_signal(1, String::from("signal1"), None)
            .unwrap();
        compiler
            .add_signal(2, String::from("signal2"), None)
            .unwrap();

        compiler.add_connection(1, 2).unwrap();
        // Connect the same node
        let result = compiler.add_connection(1, 2);

        assert!(result.is_ok());
        // No change in number of nodes
        assert_eq!(compiler.nodes.len(), 1);
    }

    #[test]
    fn test_compiler_add_connection_output_nodes() {
        let mut compiler = Compiler::new();
        compiler
            .add_signal(1, String::from("signal1"), None)
            .unwrap();
        compiler
            .add_signal(2, String::from("signal2"), None)
            .unwrap();

        // Set both nodes as output nodes
        compiler.nodes.get_mut(&1).unwrap().set_output(true);
        compiler.nodes.get_mut(&2).unwrap().set_output(true);

        let result = compiler.add_connection(1, 2);

        assert!(matches!(result, Err(CircuitError::CannotMergeOutputNodes)));
    }

    #[test]
    fn test_compiler_add_connection_constant_nodes() {
        let mut compiler = Compiler::new();
        compiler
            .add_signal(1, String::from("signal1"), Some(1))
            .unwrap();
        compiler
            .add_signal(2, String::from("signal2"), Some(2))
            .unwrap();

        let result = compiler.add_connection(1, 2);
        assert!(matches!(
            result,
            Err(CircuitError::CannotMergeConstantNodes)
        ));
    }
}
