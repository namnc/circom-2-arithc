//! # Execute Module
//!
//! This module provides functionality to handle variables execution (not signals).

use std::collections::HashMap;

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::program::ProgramError;
use crate::runtime::{DataContent, Runtime, ContextOrigin};
use crate::traverse::traverse_sequence_of_statements;
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{Access, Expression, ExpressionInfixOpcode, Statement};
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
                if res == 0 {
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
            if res == 0 {
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
            let mut name_access = String::from(var);
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let dim_u32_str = execute_expression(ac, runtime, expr, program_archive)?;
                        name_access.push('_');
                        name_access.push_str(&dim_u32_str.to_string());
                        debug!("Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(_) => {
                        todo!("Component access not handled");
                    }
                }
            }

            // Get the value of the right hand expression
            let rhe_val = execute_expression(ac, runtime, rhe, program_archive)?;

            let ctx = runtime.get_current_context()?;
            if ctx.get_data_item(&name_access).is_err() {
                ctx.declare_variable(&name_access)?;
            }
            ctx.set_data_item(&name_access, DataContent::Scalar(rhe_val))?;

            Ok(())
        }
        Statement::Return { value, .. } => {
            println!("Return expression found");
            let res = execute_expression(ac, runtime, value, program_archive)?;
            println!("RETURN {}", res);
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
) -> Result<u32, ProgramError> {
    match expression {
        Expression::Number(_, value) => Ok(value.to_u32().ok_or(ProgramError::ParsingError)?),
        Expression::InfixOp {
            lhe, infix_op, rhe, ..
        } => {
            let lhs_op = execute_expression(ac, runtime, lhe, program_archive)?;
            let rhs_op = execute_expression(ac, runtime, rhe, program_archive)?;

            Ok(execute_infix_op(&lhs_op, &rhs_op, infix_op))
        }
        Expression::Variable { name, access, .. } => {
            let mut name_access = String::from(name);
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let dim_u32 = execute_expression(ac, runtime, expr, program_archive)?;
                        name_access.push('_');
                        name_access.push_str(&dim_u32.to_string());
                        debug!("Changed var name to {}", name_access);
                    }
                    Access::ComponentAccess(_) => {
                        todo!("Component access found");
                    }
                }
            }

            let ctx = runtime.get_current_context()?;
            Ok(ctx.get_data_item(&name_access)?.get_u32()?)
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

            let mut args_map = HashMap::new();

            // We start by setting argument values to argument names
            for (_arg_name, arg_value) in arg_names.iter().zip(args) {
                // We set arg_name to have arg_value
                // Because arg_value is an expression (constant, variable, or an infix operation or a function call) we need to execute to have the actual value
                let value = execute_expression(ac, runtime, arg_value, program_archive)?;
                // We cache this to args hashmap
                args_map.insert(_arg_name, value);
            }

            // Here we need to spawn a new context for calling a function or wiring with a component (template)
            // Only after setting arguments that we can spawn a new context because the expression evaluation use values from calling context
            let _ = runtime.add_context(ContextOrigin::Call);
            let ctx = runtime.get_current_context()?;

            // Now we put args to use
            for (_arg_name, arg_value) in args_map.iter() {
                ctx.set_data_item(_arg_name, DataContent::Scalar(*(arg_value)));
            }

            let _body = if functions.contains(id) {
                program_archive.get_function_data(id).get_body_as_vec()
            } else {
                program_archive.get_template_data(id).get_body_as_vec()
            };
            
            traverse_sequence_of_statements(ac, runtime, _body, program_archive, true)?;

            // Ok(id.to_string())
            Err(ProgramError::CallError)
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
