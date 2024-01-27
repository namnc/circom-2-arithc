//! # Execute Module
//!
//! This module provides functionality to handle variables execution (not signals).

use std::collections::HashMap;

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::program::ProgramError;
use crate::runtime::{ContextOrigin, DataAccess, DataType, Runtime};
use crate::traverse::{build_access, traverse_sequence_of_statements};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{Expression, ExpressionInfixOpcode, Statement};
use circom_program_structure::program_archive::ProgramArchive;
use log::debug;

/// Executes a given statement, applying its logic or effects within the circuit's context.
pub fn execute_statement(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    statement: &Statement,
    program_archive: &ProgramArchive,
) -> Result<(), ProgramError> {
    match statement {
        Statement::InitializationBlock {
            initializations, ..
        } => {
            for stmt in initializations {
                execute_statement(ac, runtime, stmt, program_archive)?;
            }

            Ok(())
        }
        Statement::While { cond, stmt, .. } => {
            loop {
                let res = execute_expression(ac, runtime, cond, program_archive)?;
                execute_statement(ac, runtime, stmt, program_archive)?;
                if res == Some(0) {
                    break;
                }
            }
            Ok(())
        }
        Statement::IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            let res = execute_expression(ac, runtime, cond, program_archive)?;
            let else_case = else_case.as_ref().map(|e| e.as_ref());
            if res == Some(0) {
                if let Option::Some(else_stmt) = else_case {
                    execute_statement(ac, runtime, else_stmt, program_archive)
                } else {
                    Ok(())
                }
            } else {
                execute_statement(ac, runtime, if_case, program_archive)
            }
        }
        Statement::Substitution {
            var, access, rhe, ..
        } => {
            // This corresponds to a variable assignment.
            let access = build_access(runtime, ac, program_archive, var, access)?;

            // Get the value of the right hand expression
            let rhe_val = execute_expression(ac, runtime, rhe, program_archive)?;

            // Declare the variable if it is not declared yet
            let ctx = runtime.get_current_context()?;
            let declare = ctx.declare_item(DataType::Variable, &access.get_name(), &[]);
            if declare.is_ok() {
                declare?;
            }

            // Set the variable value
            ctx.set_variable(access, rhe_val)?;

            Ok(())
        }
        Statement::Return { value, .. } => {
            let access = DataAccess::new("return".to_string(), vec![]);
            let res = execute_expression(ac, runtime, value, program_archive)?;
            debug!("RETURN {:?}", res);

            let ctx = runtime.get_current_context()?;
            let declare = ctx.declare_item(DataType::Variable, &access.get_name(), &[]);

            // Added check to avoid panic when the return is already declared
            if declare.is_ok() {
                declare?;
            }

            ctx.set_variable(access, res)?;

            Ok(())
        }
        Statement::Block { stmts, .. } => {
            traverse_sequence_of_statements(ac, runtime, stmts, program_archive, true)
        }
        Statement::Declaration { .. } => {
            unreachable!("Declarations should be handled in traverse_statement")
        }
        _ => {
            unimplemented!()
        }
    }
}

/// Computes the value or effect of an expression within the circuit.
pub fn execute_expression(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    expression: &Expression,
    program_archive: &ProgramArchive,
) -> Result<Option<u32>, ProgramError> {
    match expression {
        Expression::Number(_, value) => Ok(value.to_u32()),
        Expression::InfixOp {
            lhe, infix_op, rhe, ..
        } => {
            let lhs_op = execute_expression(ac, runtime, lhe, program_archive)?
                .ok_or(ProgramError::EmptyDataItem)?;
            let rhs_op = execute_expression(ac, runtime, rhe, program_archive)?
                .ok_or(ProgramError::EmptyDataItem)?;

            Ok(Some(execute_infix_op(&lhs_op, &rhs_op, infix_op)))
        }
        Expression::Variable { name, access, .. } => {
            let access = build_access(runtime, ac, program_archive, name, access)?;
            let ctx = runtime.get_current_context()?;
            Ok(ctx.get_variable(&access)?)
        }
        Expression::Call { id, args, .. } => {
            debug!("Call found {}", id);

            // We always need to distinguish a function call from a template component wiring
            let functions = program_archive.get_function_names();
            let arg_names = if functions.contains(id) {
                program_archive.get_function_data(id).get_name_of_params()
            } else {
                program_archive.get_template_data(id).get_name_of_params()
            };

            let mut args_map: HashMap<String, u32> = HashMap::new();

            // We start by setting argument values to argument names
            for (arg_name, arg_value) in arg_names.iter().zip(args) {
                // We set arg_name to have arg_value
                // Because arg_value is an expression (constant, variable, or an infix operation or a function call) we need to execute to have the actual value
                let value = execute_expression(ac, runtime, arg_value, program_archive)?
                    .ok_or(ProgramError::EmptyDataItem)?;
                // We cache this to args hashmap
                args_map.insert(arg_name.to_string(), value);
            }

            // Here we need to spawn a new context for calling a function or wiring with a component (template)
            // Only after setting arguments that we can spawn a new context because the expression evaluation use values from calling context
            let _ = runtime.add_context(ContextOrigin::Call);
            let ctx = runtime.get_current_context()?;

            // Now we put args to use
            for (arg_name, &arg_value) in args_map.iter() {
                // TODO: Review, all items are unidimensional
                ctx.declare_item(DataType::Variable, arg_name, &[])?;
                ctx.set_variable(
                    DataAccess::new(arg_name.to_string(), vec![]),
                    Some(arg_value),
                )?;
            }

            let _body = if functions.contains(id) {
                program_archive.get_function_data(id).get_body_as_vec()
            } else {
                program_archive.get_template_data(id).get_body_as_vec()
            };

            traverse_sequence_of_statements(ac, runtime, _body, program_archive, true)?;

            if functions.contains(id) {
                // let ret = ctx.get_data_item("RETURN").unwrap().get_u32().unwrap();
                // runtime.pop_context();
                Ok(Some(0))
            } else {
                // runtime.pop_context();
                Ok(Some(0))
            }

            // Err(ProgramError::CallError)
        }
        _ => unimplemented!(),
    }
}

/// Executes an infix operation, performing the specified arithmetic or logical computation.
pub fn execute_infix_op(lhs: &u32, rhs: &u32, infix_op: &ExpressionInfixOpcode) -> u32 {
    let gate_type = AGateType::from(infix_op);
    let res = match gate_type {
        AGateType::AAdd => lhs + rhs,
        AGateType::ADiv => lhs / rhs,
        AGateType::AEq => {
            if lhs == rhs {
                1
            } else {
                0
            }
        }
        AGateType::AGEq => {
            if lhs >= rhs {
                1
            } else {
                0
            }
        }
        AGateType::AGt => {
            if lhs > rhs {
                1
            } else {
                0
            }
        }
        AGateType::ALEq => {
            if lhs <= rhs {
                1
            } else {
                0
            }
        }
        AGateType::ALt => {
            if lhs < rhs {
                1
            } else {
                0
            }
        }
        AGateType::AMul => lhs * rhs,
        AGateType::ANeq => {
            if lhs != rhs {
                1
            } else {
                0
            }
        }
        AGateType::ANone => unimplemented!(),
        AGateType::ASub => lhs - rhs,
    };

    debug!("Execute Infix Op: {} {} {} = {}", lhs, gate_type, rhs, res);
    res
}
