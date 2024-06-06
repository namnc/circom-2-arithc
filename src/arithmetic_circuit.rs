use std::{
    io::{BufRead, Read, Write},
    str::FromStr,
};

use crate::compiler::{ArithmeticGate, CircuitError};

pub struct ArithmeticCircuit {
    wire_count: u32,
    inputs: Vec<String>,
    outputs: Vec<String>,
    gates: Vec<ArithmeticGate>,
}

impl ArithmeticCircuit {
    pub fn write_bristol<W: Write>(&self, w: &mut W) -> Result<(), CircuitError> {
        writeln!(w, "{} {}", self.gates.len(), self.wire_count)?;

        write!(w, "{}", self.inputs.len())?;

        for _ in 0..self.inputs.len() {
            write!(w, " 1")?;
        }

        writeln!(w)?;

        write!(w, "{}", self.outputs.len())?;

        for _ in 0..self.outputs.len() {
            write!(w, " 1")?;
        }

        writeln!(w)?;

        for gate in &self.gates {
            writeln!(
                w,
                "2 1 {} {} {} {:?}",
                gate.lh_in, gate.rh_in, gate.out, gate.op
            )?;
        }

        Ok(())
    }

    pub fn read_bristol<R: Read + BufRead>(r: &mut R) -> Result<ArithmeticCircuit, CircuitError> {
        let (gate_count, wire_count) = BristolLine::read(r)?.circuit_sizes()?;

        let mut inputs = Vec::new();
        for i in 0..BristolLine::read(r)?.io_count()? {
            inputs.push(format!("input{}", i));
        }

        let mut outputs = Vec::new();
        for i in 0..BristolLine::read(r)?.io_count()? {
            outputs.push(format!("output{}", i));
        }

        let mut gates = Vec::new();
        for _ in 0..gate_count {
            gates.push(BristolLine::read(r)?.gate()?);
        }

        for line in r.lines() {
            if !line?.trim().is_empty() {
                return Err(CircuitError::InvalidInput {
                    message: "Unexpected non-whitespace line after gates".into(),
                });
            }
        }

        Ok(ArithmeticCircuit {
            wire_count,
            inputs,
            outputs,
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
            return Err(CircuitError::InvalidInput {
                message: format!("Expected {} parts", count + 1),
            });
        }

        for i in 1..self.0.len() {
            if self.get_str(i)? != "1" {
                return Err(CircuitError::InvalidInput {
                    message: format!("Expected 1 at index {}", i),
                });
            }
        }

        Ok(count)
    }

    pub fn gate(&self) -> Result<ArithmeticGate, CircuitError> {
        if self.0.len() != 6 {
            return Err(CircuitError::InvalidInput {
                message: "Expected 6 parts".into(),
            });
        }

        if self.get::<u32>(0)? != 2 || self.get::<u32>(1)? != 1 {
            return Err(CircuitError::InvalidInput {
                message: "Expected 2 inputs and 1 output".into(),
            });
        }

        Ok(ArithmeticGate {
            lh_in: self.get(2)?,
            rh_in: self.get(3)?,
            out: self.get(4)?,
            op: serde_json::from_str(self.get_str(5)?)?,
        })
    }

    fn get<T: FromStr>(&self, index: usize) -> Result<T, CircuitError> {
        self.0
            .get(index)
            .ok_or(CircuitError::InvalidInput {
                message: format!("Index {} out of bounds", index),
            })?
            .parse::<T>()
            .map_err(|_| CircuitError::InvalidInput {
                message: format!("Failed to convert at index {}", index),
            })
    }

    fn get_str(&self, index: usize) -> Result<&str, CircuitError> {
        self.0
            .get(index)
            .ok_or(CircuitError::InvalidInput {
                message: format!("Index {} out of bounds", index),
            })
            .map(|s| s.as_str())
    }
}
