//! # Execute Module
//!
//! This module provides functionality to handle variables execution (not signals).

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::program::ProgramError;
use crate::runtime::{DataContent, Runtime};
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
                let var = String::from("while");
                let res = execute_expression(ac, runtime, &var, cond, program_archive)?;
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
            let var = String::from("IFTHENELSE");
            let res = execute_expression(ac, runtime, &var, cond, program_archive)?;
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
            debug!("Assigning value to variable: {}", var.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let dim_u32_str =
                            execute_expression(ac, runtime, var, expr, program_archive)?;
                        name_access.push('_');
                        name_access.push_str(&dim_u32_str.to_string());
                        debug!("Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(_) => {
                        todo!("Component access not handled");
                    }
                }
            }
            let rhs = execute_expression(ac, runtime, &name_access, rhe, program_archive)?;
            debug!("Assigning {} to {}", rhs, &name_access);
            let ctx = runtime.get_current_context()?;
            let res = ctx.get_data_item(&name_access);

            match res {
                Ok(data_item) => {
                    if let Some(val) = data_item.get_content() {
                        ctx.set_data_item(&name_access, val.clone())?;
                    } else {
                        ctx.set_data_item(&name_access, DataContent::Scalar(rhs))?;
                    }
                    Ok(())
                }
                Err(_) => {
                    ctx.declare_variable(&name_access)?;
                    ctx.set_data_item(&name_access, DataContent::Scalar(rhs))?;
                    Ok(())
                }
            }
        }
        Statement::Return { value, .. } => {
            println!("Return expression found");
            let var = String::from("RETURN");
            let res = execute_expression(ac, runtime, &var, value, program_archive)?;
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
    _var: &String,
    expression: &Expression,
    program_archive: &ProgramArchive,
) -> Result<u32, ProgramError> {
    match expression {
        Expression::Number(_, value) => {
            // Declaring a constant.
            let val = value.to_u32().ok_or(ProgramError::ParsingError)?;

            let res = runtime.get_current_context()?.declare_const(val);

            if res.is_ok() {
                // Add constant to circuit.
                ac.add_const_var(val, val);
            }

            Ok(val)
        }
        Expression::InfixOp {
            lhe, infix_op, rhe, ..
        } => {
            let ctx = runtime.get_current_context()?;

            //TODO: for generic handling we should generate a name for an intermediate expression, we could ideally use only the values returned
            let varlhs = ctx.declare_auto_var()?;
            let varrhs = ctx.declare_auto_var()?;

            let varlop = execute_expression(ac, runtime, &varlhs, lhe, program_archive)?;
            let varrop = execute_expression(ac, runtime, &varrhs, rhe, program_archive)?;

            let res = execute_infix_op(&varlop, &varrop, infix_op);
            debug!("execute_infix_op res {}", res);

            Ok(res)
        }
        Expression::Variable { name, access, .. } => {
            let mut name_access = String::from(name);
            debug!("Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let dim_u32_str =
                            execute_expression(ac, runtime, _var, expr, program_archive)?;
                        name_access.push('_');
                        name_access.push_str(&dim_u32_str.to_string());
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
            println!("Call found {}", id);

            // HERE IS CODE FOR ARGUMENTS
            // TODO: HERE WE SHOULD NOT HAVE TEMPLATE CALL
            let functions = program_archive.get_function_names();
            let arg_names = if functions.contains(id) {
                program_archive.get_function_data(id).get_name_of_params()
            } else {
                program_archive.get_template_data(id).get_name_of_params()
            };

            for (arg_name, arg_value) in arg_names.iter().zip(args) {
                // We set arg_name to have arg_value
                let _ = execute_expression(ac, runtime, arg_name, arg_value, program_archive);
                // TODO: set res to arg_name
            }

            // HERE IS CODE FOR FUNCTIGON

            let fn_body = program_archive.get_function_data(id).get_body_as_vec();
            traverse_sequence_of_statements(ac, runtime, fn_body, program_archive, true)?;

            // HERE IS CODE FOR TEMPLATE

            // find the template and execute it
            // let template_body = program_archive.get_template_data(id).get_body_as_vec();

            // traverse_sequence_of_statements(
            //     ac,
            //     runtime,
            //     template_body,
            //     program_archive,
            //     true,
            // );
            Err(ProgramError::CallError)
        }
        _ => unimplemented!(),
    }
}

/// Executes an infix operation, performing the specified arithmetic or logical computation.
pub fn execute_infix_op(lhs: &u32, rhs: &u32, infix_op: &ExpressionInfixOpcode) -> u32 {
    match AGateType::from(infix_op) {
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
    }
}
