#![allow(clippy::upper_case_acronyms)]

use bristol_circuit::BristolCircuit;
use circom_2_arithc::{a_gate_type::AGateType, cli::ValueType};
use sim_circuit::{
    circuit::{CircuitBuilder, CircuitMemory, GenericCircuit, GenericCircuitExecutor},
    model::{Component, Executable, Memory},
};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
enum ArithmeticOperation {
    ADD,
    DIV,
    EQ,
    GEQ,
    GT,
    LEQ,
    LT,
    MUL,
    NEQ,
    SUB,
    XOR,
    POW,
    INTDIV,
    MOD,
    SHIFTL,
    SHIFTR,
    BOOLOR,
    BOOLAND,
    BITOR,
    BITAND,
}

impl From<AGateType> for ArithmeticOperation {
    fn from(gate_type: AGateType) -> Self {
        match gate_type {
            AGateType::AAdd => ArithmeticOperation::ADD,
            AGateType::ADiv => ArithmeticOperation::DIV,
            AGateType::AEq => ArithmeticOperation::EQ,
            AGateType::AGEq => ArithmeticOperation::GEQ,
            AGateType::AGt => ArithmeticOperation::GT,
            AGateType::ALEq => ArithmeticOperation::LEQ,
            AGateType::ALt => ArithmeticOperation::LT,
            AGateType::AMul => ArithmeticOperation::MUL,
            AGateType::ANeq => ArithmeticOperation::NEQ,
            AGateType::ASub => ArithmeticOperation::SUB,
            AGateType::AXor => ArithmeticOperation::XOR,
            AGateType::APow => ArithmeticOperation::POW,
            AGateType::AIntDiv => ArithmeticOperation::INTDIV,
            AGateType::AMod => ArithmeticOperation::MOD,
            AGateType::AShiftL => ArithmeticOperation::SHIFTL,
            AGateType::AShiftR => ArithmeticOperation::SHIFTR,
            AGateType::ABoolOr => ArithmeticOperation::BOOLOR,
            AGateType::ABoolAnd => ArithmeticOperation::BOOLAND,
            AGateType::ABitOr => ArithmeticOperation::BITOR,
            AGateType::ABitAnd => ArithmeticOperation::BITAND,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct ArithmeticGate {
    operation: ArithmeticOperation,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

impl Component for ArithmeticGate {
    fn inputs(&self) -> &[usize] {
        &self.inputs
    }

    fn outputs(&self) -> &[usize] {
        &self.outputs
    }

    fn set_inputs(&mut self, inputs: Vec<usize>) {
        self.inputs = inputs;
    }

    fn set_outputs(&mut self, outputs: Vec<usize>) {
        self.outputs = outputs;
    }
}

impl Executable<u32, CircuitMemory<u32>> for ArithmeticGate {
    type Error = ();

    fn execute(&self, memory: &mut CircuitMemory<u32>) -> Result<(), Self::Error> {
        let a = memory.read(self.inputs[0]).unwrap();
        let b = memory.read(self.inputs[1]).unwrap();

        let result = match self.operation {
            ArithmeticOperation::ADD => a + b,
            ArithmeticOperation::DIV => a / b,
            ArithmeticOperation::EQ => (a == b) as u32,
            ArithmeticOperation::GEQ => (a >= b) as u32,
            ArithmeticOperation::GT => (a > b) as u32,
            ArithmeticOperation::LEQ => (a <= b) as u32,
            ArithmeticOperation::LT => (a < b) as u32,
            ArithmeticOperation::MUL => a * b,
            ArithmeticOperation::NEQ => (a != b) as u32,
            ArithmeticOperation::SUB => a - b,
            ArithmeticOperation::XOR => a ^ b,
            ArithmeticOperation::POW => a.pow(b),
            ArithmeticOperation::INTDIV => a / b,
            ArithmeticOperation::MOD => a % b,
            ArithmeticOperation::SHIFTL => a << b,
            ArithmeticOperation::SHIFTR => a >> b,
            ArithmeticOperation::BOOLOR => (a != 0 || b != 0) as u32,
            ArithmeticOperation::BOOLAND => (a != 0 && b != 0) as u32,
            ArithmeticOperation::BITOR => a | b,
            ArithmeticOperation::BITAND => a & b,
        };

        memory.write(self.outputs[0], result).unwrap();
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArithmeticCircuit {
    gates: Vec<ArithmeticGate>,
    constants: HashMap<usize, u32>,
    label_to_index: HashMap<String, usize>,
    input_indices: Vec<usize>,
    outputs: Vec<String>,
}

impl ArithmeticCircuit {
    /// Create a new `ArithmeticCircuit` from a bristol circuit
    pub fn new_from_bristol(circuit: BristolCircuit) -> Result<Self, &'static str> {
        let mut label_to_index: HashMap<String, usize> = HashMap::new();
        let mut outputs: Vec<String> = Vec::new();
        let mut input_indices: Vec<usize> = Vec::new();
        let mut gates: Vec<ArithmeticGate> = Vec::new();

        // Get circuit inputs
        let inputs = circuit.info.input_name_to_wire_index;
        for (label, index) in inputs {
            label_to_index.insert(label, index);
            input_indices.push(index);
        }

        // Get circuit constants
        let mut constants: HashMap<usize, u32> = HashMap::new();
        for (_, constant_info) in circuit.info.constants {
            input_indices.push(constant_info.wire_index);
            constants.insert(
                constant_info.wire_index,
                constant_info.value.parse().unwrap(),
            );
        }

        // Get circuit outputs
        let output_map = circuit.info.output_name_to_wire_index;
        let mut output_indices = vec![];
        for (label, index) in output_map {
            label_to_index.insert(label.clone(), index);
            outputs.push(label);
            output_indices.push(index);
        }

        // Transform and add gates
        for gate in circuit.gates {
            let operation = ArithmeticOperation::from(
                gate.op
                    .parse::<AGateType>()
                    .map_err(|_| "unrecognized gate")?,
            );

            let arithmetic_gate = ArithmeticGate {
                operation,
                inputs: gate.inputs,
                outputs: gate.outputs,
            };
            gates.push(arithmetic_gate);
        }

        Ok(Self {
            gates,
            constants,
            label_to_index,
            input_indices,
            outputs,
        })
    }

    /// Run the circuit
    pub fn run(&self, inputs: HashMap<String, u32>) -> Result<HashMap<String, u32>, &'static str> {
        // Build circuit
        let circuit = self.build_circuit();
        // Instantiate a circuit executor
        let mut executor: GenericCircuitExecutor<ArithmeticGate, u32> =
            GenericCircuitExecutor::new(circuit);

        // The executor receives a map of WireIndex -> Value
        let input_map: HashMap<usize, u32> = inputs
            .iter()
            .map(|(label, value)| {
                let index = self
                    .label_to_index
                    .get(label)
                    .ok_or("Input label not found")
                    .unwrap();
                (*index, *value)
            })
            .collect();

        // Load constants into the input map
        let input_map = self
            .constants
            .iter()
            .fold(input_map, |mut acc, (index, value)| {
                acc.insert(*index, *value);
                acc
            });

        let output = executor.run(&input_map).unwrap();

        // The executor returns a map of WireIndex -> Value
        let output_map: HashMap<String, u32> = self
            .outputs
            .iter()
            .map(|label| {
                let index = self
                    .label_to_index
                    .get(label)
                    .ok_or("Output label not found")
                    .unwrap();
                (label.clone(), *output.get(index).unwrap())
            })
            .collect();

        Ok(output_map)
    }

    fn build_circuit(&self) -> GenericCircuit<ArithmeticGate, u32> {
        let mut builder = CircuitBuilder::<ArithmeticGate, u32>::new();
        builder.add_inputs(&self.input_indices);

        for gate in &self.gates {
            builder.add_component(gate.clone()).unwrap();
        }

        builder.build().unwrap()
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use bristol_circuit::ConstantInfo;
    use circom_2_arithc::{cli::Args, program::compile};

    fn simulation_test(
        circuit_path: &str,
        inputs: &[(&str, u32)],
        expected_outputs: &[(&str, u32)],
    ) {
        let compiler_input = Args::new(circuit_path.into(), "./".into(), ValueType::Sint, None);
        let circuit = compile(&compiler_input).unwrap().build_circuit().unwrap();
        let arithmetic_circuit = ArithmeticCircuit::new_from_bristol(circuit).unwrap();

        let mut input_map: HashMap<String, u32> = HashMap::new();
        for (label, value) in inputs {
            input_map.insert(label.to_string(), *value);
        }

        let outputs: HashMap<String, u32> = arithmetic_circuit.run(input_map).unwrap();

        for (label, expected_value) in expected_outputs {
            let value = outputs.get(*label).unwrap();
            assert_eq!(value, expected_value);
        }
    }

    #[test]
    fn test_add_zero() {
        simulation_test(
            "tests/circuits/integration/addZero.circom",
            &[("0.in", 42)],
            &[("0.out", 42)],
        );
    }

    #[test]
    fn test_infix_ops() {
        simulation_test(
            "tests/circuits/integration/infixOps.circom",
            &[
                ("0.x0", 0),
                ("0.x1", 1),
                ("0.x2", 2),
                ("0.x3", 3),
                ("0.x4", 4),
                ("0.x5", 5),
            ],
            &[
                ("0.mul_2_3", 6),
                // ("0.div_4_3", 1), // unsupported for NumberU32
                ("0.idiv_4_3", 1),
                ("0.add_3_4", 7),
                ("0.sub_4_1", 3),
                ("0.pow_2_4", 16),
                ("0.mod_5_3", 2),
                ("0.shl_5_1", 10),
                ("0.shr_5_1", 2),
                ("0.leq_2_3", 1),
                ("0.leq_3_3", 1),
                ("0.leq_4_3", 0),
                ("0.geq_2_3", 0),
                ("0.geq_3_3", 1),
                ("0.geq_4_3", 1),
                ("0.lt_2_3", 1),
                ("0.lt_3_3", 0),
                ("0.lt_4_3", 0),
                ("0.gt_2_3", 0),
                ("0.gt_3_3", 0),
                ("0.gt_4_3", 1),
                ("0.eq_2_3", 0),
                ("0.eq_3_3", 1),
                ("0.neq_2_3", 1),
                ("0.neq_3_3", 0),
                ("0.or_0_1", 1),
                ("0.and_0_1", 0),
                ("0.bit_or_1_3", 3),
                ("0.bit_and_1_3", 1),
                ("0.bit_xor_1_3", 2),
            ],
        );
    }

    #[test]
    fn test_matrix_element_multiplication() {
        simulation_test(
            "tests/circuits/integration/matElemMul.circom",
            &[
                ("0.a[0][0]", 2),
                ("0.a[0][1]", 2),
                ("0.a[1][0]", 2),
                ("0.a[1][1]", 2),
                ("0.b[0][0]", 2),
                ("0.b[0][1]", 2),
                ("0.b[1][0]", 2),
                ("0.b[1][1]", 2),
            ],
            &[
                ("0.out[0][0]", 4),
                ("0.out[0][1]", 4),
                ("0.out[1][0]", 4),
                ("0.out[1][1]", 4),
            ],
        );
    }

    #[test]
    fn test_sum() {
        simulation_test(
            "tests/circuits/integration/sum.circom",
            &[("0.a", 3), ("0.b", 5)],
            &[("0.out", 8)],
        );
    }

    #[test]
    fn test_x_eq_x() {
        simulation_test(
            "tests/circuits/integration/xEqX.circom",
            &[("0.x", 37)],
            &[("0.out", 1)],
        );
    }

    #[test]
    fn test_out_of_bounds() {
        let compiler_input = Args::new(
            "tests/circuits/integration/indexOutOfBounds.circom".into(),
            "./".into(),
            ValueType::Sint,
            None,
        );
        let circuit = compile(&compiler_input);

        assert!(circuit.is_err());
        assert_eq!(
            circuit.unwrap_err().to_string(),
            "Runtime error: Index out of bounds"
        );
    }

    #[test]
    fn test_constant_sum() {
        let compiler_input = Args::new(
            "tests/circuits/integration/constantSum.circom".into(),
            "./".into(),
            ValueType::Sint,
            None,
        );
        let circuit_res = compile(&compiler_input);

        assert!(circuit_res.is_ok());

        let circuit = circuit_res.unwrap().build_circuit().unwrap();

        assert_eq!(circuit.info.constants.len(), 1);
        assert_eq!(
            circuit.info.constants.get("0.const_signal_8_1"),
            Some(&ConstantInfo {
                value: "8".to_string(), // 5 + 3
                wire_index: 0
            })
        );
    }

    #[test]
    fn test_direct_output() {
        let compiler_input = Args::new(
            "tests/circuits/integration/directOutput.circom".into(),
            "./".into(),
            ValueType::Sint,
            None,
        );
        let circuit_res = compile(&compiler_input);

        assert!(circuit_res.is_ok());

        let circuit = circuit_res.unwrap().build_circuit().unwrap();

        let expected_output = HashMap::from([("0.out".to_string(), 0)]);
        assert_eq!(circuit.info.output_name_to_wire_index, expected_output);
        assert_eq!(circuit.info.constants.len(), 1);
        assert_eq!(
            circuit.info.constants.get("0.const_signal_42_1"),
            Some(&ConstantInfo {
                value: "42".to_string(),
                wire_index: 0
            })
        );
    }

    #[ignore]
    #[test]
    fn test_under_constrained() {
        // FIXME: There should be an error instead (zero comes from default initialization, not from
        //        running the circuit)
        simulation_test(
            "tests/circuits/integration/underConstrained.circom",
            &[],
            &[("0.x", 0)],
        );
    }

    #[ignore]
    #[test]
    fn test_prefix_ops() {
        // FIXME: The compiler sees several of the outputs as inputs, leading to the error below
        //        CircuitError(Inconsistency {
        //            message: "Node 10 used for both input 0.complementC and output 0.complementC"
        //        })
        simulation_test(
            "tests/circuits/integration/prefixOps.circom",
            &[("0.a", 0), ("0.b", 1), ("0.c", 2)],
            &[
                ("0.negateA", 0),                                          // -0
                ("0.notA", 1),                                             // !0
                ("0.notB", 0),                                             // !1
                ("0.notC", 0),                                             // !2
                ("0.complementA", 0b_11111111_11111111_11111111_11111111), // ~0
                ("0.complementB", 0b_11111111_11111111_11111111_11111110), // ~1
                ("0.complementC", 0b_11111111_11111111_11111111_11111101), // ~2
            ],
        );
    }
}
