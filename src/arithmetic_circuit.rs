use crate::compiler::{ArithmeticGate, CircuitError};
use circom_program_structure::ast::ExpressionInfixOpcode;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Write},
    str::FromStr,
};
use strum_macros::{Display as StrumDisplay, EnumString};

/// The supported Arithmetic gate types.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, EnumString, StrumDisplay)]
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
    APow,
    AIntDiv,
    AMod,
    AShiftL,
    AShiftR,
    ABoolOr,
    ABoolAnd,
    ABitOr,
    ABitAnd,
}

impl From<&ExpressionInfixOpcode> for AGateType {
    fn from(opcode: &ExpressionInfixOpcode) -> Self {
        match opcode {
            ExpressionInfixOpcode::Mul => AGateType::AMul,
            ExpressionInfixOpcode::Div => AGateType::ADiv,
            ExpressionInfixOpcode::Add => AGateType::AAdd,
            ExpressionInfixOpcode::Sub => AGateType::ASub,
            ExpressionInfixOpcode::Pow => AGateType::APow,
            ExpressionInfixOpcode::IntDiv => AGateType::AIntDiv,
            ExpressionInfixOpcode::Mod => AGateType::AMod,
            ExpressionInfixOpcode::ShiftL => AGateType::AShiftL,
            ExpressionInfixOpcode::ShiftR => AGateType::AShiftR,
            ExpressionInfixOpcode::LesserEq => AGateType::ALEq,
            ExpressionInfixOpcode::GreaterEq => AGateType::AGEq,
            ExpressionInfixOpcode::Lesser => AGateType::ALt,
            ExpressionInfixOpcode::Greater => AGateType::AGt,
            ExpressionInfixOpcode::Eq => AGateType::AEq,
            ExpressionInfixOpcode::NotEq => AGateType::ANeq,
            ExpressionInfixOpcode::BoolOr => AGateType::ABoolOr,
            ExpressionInfixOpcode::BoolAnd => AGateType::ABoolAnd,
            ExpressionInfixOpcode::BitOr => AGateType::ABitOr,
            ExpressionInfixOpcode::BitAnd => AGateType::ABitAnd,
            ExpressionInfixOpcode::BitXor => AGateType::AXor,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArithmeticCircuit {
    pub wire_count: u32,
    pub info: CircuitInfo,
    pub gates: Vec<ArithmeticGate>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitInfo {
    pub input_name_to_wire_index: HashMap<String, u32>,
    pub constants: HashMap<String, ConstantInfo>,
    pub output_name_to_wire_index: HashMap<String, u32>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantInfo {
    pub value: String,
    pub wire_index: u32,
}

impl ArithmeticCircuit {
    pub fn get_bristol_string(&self) -> Result<String, CircuitError> {
        let mut output = Vec::new();
        let mut writer = BufWriter::new(&mut output);

        self.write_bristol(&mut writer)?;
        drop(writer);

        String::from_utf8(output).map_err(|_| CircuitError::ParsingError {
            message: "Generated Bristol data was not valid utf8".into(),
        })
    }

    pub fn from_info_and_bristol_string(
        info: CircuitInfo,
        input: &str,
    ) -> Result<ArithmeticCircuit, CircuitError> {
        ArithmeticCircuit::read_info_and_bristol(info, &mut BufReader::new(input.as_bytes()))
    }

    pub fn write_bristol<W: Write>(&self, w: &mut W) -> Result<(), CircuitError> {
        writeln!(w, "{} {}", self.gates.len(), self.wire_count)?;

        write!(w, "{}", self.info.input_name_to_wire_index.len())?;

        for _ in 0..self.info.input_name_to_wire_index.len() {
            write!(w, " 1")?;
        }

        writeln!(w)?;

        write!(w, "{}", self.info.output_name_to_wire_index.len())?;

        for _ in 0..self.info.output_name_to_wire_index.len() {
            write!(w, " 1")?;
        }

        writeln!(w)?;
        writeln!(w)?;

        for gate in &self.gates {
            writeln!(
                w,
                "2 1 {} {} {} {}",
                gate.lh_in, gate.rh_in, gate.out, gate.op
            )?;
        }

        Ok(())
    }

    pub fn read_info_and_bristol<R: BufRead>(
        info: CircuitInfo,
        r: &mut R,
    ) -> Result<ArithmeticCircuit, CircuitError> {
        let (gate_count, wire_count) = BristolLine::read(r)?.circuit_sizes()?;

        let input_count = BristolLine::read(r)?.io_count()?;
        if input_count != info.input_name_to_wire_index.len() as u32 {
            return Err(CircuitError::Inconsistency {
                message: "Input count mismatch".into(),
            });
        }

        let output_count = BristolLine::read(r)?.io_count()?;
        if output_count != info.output_name_to_wire_index.len() as u32 {
            return Err(CircuitError::Inconsistency {
                message: "Output count mismatch".into(),
            });
        }

        let mut gates = Vec::new();
        for _ in 0..gate_count {
            gates.push(BristolLine::read(r)?.gate()?);
        }

        for line in r.lines() {
            if !line?.trim().is_empty() {
                return Err(CircuitError::ParsingError {
                    message: "Unexpected non-whitespace line after gates".into(),
                });
            }
        }

        Ok(ArithmeticCircuit {
            wire_count,
            info,
            gates,
        })
    }
}

struct BristolLine(Vec<String>);

impl BristolLine {
    pub fn read(r: &mut impl BufRead) -> Result<Self, CircuitError> {
        loop {
            let mut line = String::new();
            r.read_line(&mut line)?;

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            return Ok(BristolLine(
                line.split_whitespace()
                    .map(|part| part.to_string())
                    .collect(),
            ));
        }
    }

    pub fn circuit_sizes(&self) -> Result<(u32, u32), CircuitError> {
        Ok((self.get(0)?, self.get(1)?))
    }

    pub fn io_count(&self) -> Result<u32, CircuitError> {
        let count = self.get::<u32>(0)?;

        if self.0.len() != (count + 1) as usize {
            return Err(CircuitError::ParsingError {
                message: format!("Expected {} parts", count + 1),
            });
        }

        for i in 1..self.0.len() {
            if self.get_str(i)? != "1" {
                return Err(CircuitError::ParsingError {
                    message: format!("Expected 1 at index {}", i),
                });
            }
        }

        Ok(count)
    }

    pub fn gate(&self) -> Result<ArithmeticGate, CircuitError> {
        if self.0.len() != 6 {
            return Err(CircuitError::ParsingError {
                message: "Expected 6 parts".into(),
            });
        }

        if self.get::<u32>(0)? != 2 || self.get::<u32>(1)? != 1 {
            return Err(CircuitError::ParsingError {
                message: "Expected 2 inputs and 1 output".into(),
            });
        }

        Ok(ArithmeticGate {
            lh_in: self.get(2)?,
            rh_in: self.get(3)?,
            out: self.get(4)?,
            op: self.get(5)?,
        })
    }

    fn get<T: FromStr>(&self, index: usize) -> Result<T, CircuitError> {
        self.0
            .get(index)
            .ok_or(CircuitError::ParsingError {
                message: format!("Index {} out of bounds", index),
            })?
            .parse::<T>()
            .map_err(|_| CircuitError::ParsingError {
                message: format!("Failed to convert at index {}", index),
            })
    }

    fn get_str(&self, index: usize) -> Result<&str, CircuitError> {
        self.0
            .get(index)
            .ok_or(CircuitError::ParsingError {
                message: format!("Index {} out of bounds", index),
            })
            .map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    // Helper function to create a sample ArithmeticCircuit
    fn create_sample_circuit() -> ArithmeticCircuit {
        ArithmeticCircuit {
            // d = (a + b) * b
            // we need to use inputX and outputX to match deserialization from bristol format
            // which doesn't specify the wire names
            wire_count: 4,
            info: CircuitInfo {
                input_name_to_wire_index: [("input0".to_string(), 0), ("input1".to_string(), 1)]
                    .iter()
                    .cloned()
                    .collect(),
                constants: Default::default(),
                output_name_to_wire_index: [("output0".to_string(), 3)].iter().cloned().collect(),
            },
            gates: vec![
                ArithmeticGate {
                    lh_in: 0,
                    rh_in: 1,
                    out: 2,
                    op: AGateType::AAdd,
                },
                ArithmeticGate {
                    lh_in: 2,
                    rh_in: 1,
                    out: 3,
                    op: AGateType::AMul,
                },
            ],
        }
    }

    fn clean(src: &str) -> String {
        src.trim_start()
            .trim_end_matches(char::is_whitespace)
            .lines()
            .map(str::trim)
            .collect::<Vec<&str>>()
            .join("\n")
            + "\n"
    }

    #[test]
    fn test_write_bristol() {
        assert_eq!(
            create_sample_circuit().get_bristol_string().unwrap(),
            clean(
                "
                    2 4
                    2 1 1
                    1 1
                    
                    2 1 0 1 2 AAdd
                    2 1 2 1 3 AMul
                ",
            ),
        );
    }

    #[test]
    fn test_read_bristol() {
        assert_eq!(
            ArithmeticCircuit::from_info_and_bristol_string(
                CircuitInfo {
                    input_name_to_wire_index: [
                        ("input0".to_string(), 0),
                        ("input1".to_string(), 1)
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    constants: Default::default(),
                    output_name_to_wire_index: [("output0".to_string(), 3)]
                        .iter()
                        .cloned()
                        .collect(),
                },
                "
                    2 4
                    2 1 1
                    1 1

                    2 1 0 1 2 AAdd
                    2 1 2 1 3 AMul
                "
            )
            .unwrap(),
            create_sample_circuit()
        );
    }

    #[test]
    fn test_bristol_line_read() {
        let input_data = "2 4\n";
        let mut reader = BufReader::new(Cursor::new(input_data));

        let bristol_line = BristolLine::read(&mut reader).unwrap();
        assert_eq!(bristol_line.0, vec!["2", "4"]);
    }

    #[test]
    fn test_bristol_line_circuit_sizes() {
        let bristol_line = BristolLine(vec!["2".to_string(), "4".to_string()]);
        let (gate_count, wire_count) = bristol_line.circuit_sizes().unwrap();
        assert_eq!(gate_count, 2);
        assert_eq!(wire_count, 4);
    }

    #[test]
    fn test_bristol_line_io_count() {
        let bristol_line = BristolLine(vec!["2".to_string(), "1".to_string(), "1".to_string()]);
        let io_count = bristol_line.io_count().unwrap();
        assert_eq!(io_count, 2);
    }

    #[test]
    fn test_bristol_line_gate() {
        let bristol_line = BristolLine(vec![
            "2".to_string(),
            "1".to_string(),
            "0".to_string(),
            "1".to_string(),
            "2".to_string(),
            "AAdd".to_string(),
        ]);
        let gate = bristol_line.gate().unwrap();
        assert_eq!(gate.lh_in, 0);
        assert_eq!(gate.rh_in, 1);
        assert_eq!(gate.out, 2);
        assert_eq!(gate.op, AGateType::AAdd);
    }

    #[test]
    fn test_bristol_line_get() {
        let bristol_line = BristolLine(vec!["2".to_string(), "4".to_string()]);
        let value: u32 = bristol_line.get(0).unwrap();
        assert_eq!(value, 2);
    }

    #[test]
    fn test_bristol_line_get_str() {
        let bristol_line = BristolLine(vec!["2".to_string(), "4".to_string()]);
        let value = bristol_line.get_str(1).unwrap();
        assert_eq!(value, "4");
    }
}
