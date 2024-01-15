//! # Traverse Module
//!
//! This module provides functionality for traversing statements, expressions, infix operations and declaration of components, signals and variables.
//!
//! It's main purpose is to traverse signals.

use std::collections::HashMap;

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::execute::{execute_expression, execute_infix_op, execute_statement};
use crate::program::ProgramError;
use crate::runtime::{DataContent, DataType, Runtime, ContextOrigin};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{
    Access, Expression, ExpressionInfixOpcode, Statement, VariableType,
};
use circom_program_structure::program_archive::ProgramArchive;
use log::debug;

/// Processes a sequence of statements, handling each based on its specific type and context.
pub fn traverse_sequence_of_statements(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    statements: &[Statement],
    program_archive: &ProgramArchive,
    _is_complete_template: bool,
) -> Result<(), ProgramError> {
    for statement in statements {
        traverse_statement(ac, runtime, statement, program_archive)?;
    }
    // TODO: handle complete template

    Ok(())
}

/// Analyzes a single statement, delegating to specialized functions based on the statement's nature.
pub fn traverse_statement(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    stmt: &Statement,
    program_archive: &ProgramArchive,
) -> Result<(), ProgramError> {
    match stmt {
        Statement::InitializationBlock {
            initializations, ..
        } => {
            for statement in initializations {
                traverse_statement(ac, runtime, statement, program_archive)?;
            }

            Ok(())
        }
        Statement::Declaration {
            xtype,
            name,
            dimensions,
            ..
        } => {
            // Process index in case of array
            let dim_u32_vec: Vec<u32> = dimensions
                .iter()
                .map(|dimension| execute_expression(ac, runtime, dimension, program_archive))
                .collect::<Result<Vec<u32>, _>>()?;

            match xtype {
                VariableType::Component => {
                    todo!("Component declaration not handled")
                }
                VariableType::Var => traverse_declaration(ac, runtime, name, xtype, &dim_u32_vec),
                VariableType::Signal(_, _) => {
                    traverse_declaration(ac, runtime, name, xtype, &dim_u32_vec)
                }
                _ => unimplemented!(),
            }
        }
        Statement::While { cond, stmt, .. } => {
            loop {
                let result = execute_expression(ac, runtime, cond, program_archive)?;
                if result == 0 {
                    break;
                }

                debug!("While res = {}", result);
                traverse_statement(ac, runtime, stmt, program_archive)?
            }

            Ok(())
        }
        Statement::IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            let result = execute_expression(ac, runtime, cond, program_archive)?;
            let else_case = else_case.as_ref().map(|e| e.as_ref());
            if result == 0 {
                if let Option::Some(else_stmt) = else_case {
                    return traverse_statement(ac, runtime, else_stmt, program_archive);
                }
                Ok(())
            } else {
                traverse_statement(ac, runtime, if_case, program_archive)
            }
        }
        Statement::Substitution {
            var, access, rhe, ..
        } => {
            debug!("Substitution for {}", var.to_string());
            let mut name_access = String::from(var);
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Sub Array access found");
                        let dim_u32_str = traverse_expression(ac, runtime, expr, program_archive)?;
                        name_access.push('_');
                        name_access.push_str(&dim_u32_str);
                        debug!("Sub Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(_) => {
                        todo!("Sub Component access not handled");
                    }
                }
            }

            // Check if we're dealing with a signal or a variable
            let ctx = runtime.get_current_context()?;
            let data_item = ctx.get_data_item(&name_access);
            if let Ok(data_value) = data_item {
                match data_value.get_data_type() {
                    DataType::Signal => {
                        traverse_expression(ac, runtime, rhe, program_archive)?;
                    }
                    DataType::Variable => {
                        execute_statement(ac, runtime, stmt, program_archive)?;
                    }
                }
            }
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
        _ => unimplemented!("Statement not implemented"),
    }
}

/// Process an expression and returns a name of a variable that contains the result.
pub fn traverse_expression(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    expression: &Expression,
    _program_archive: &ProgramArchive,
) -> Result<String, ProgramError> {
    match expression {
        Expression::Number(_, value) => {
            let num_val = value.to_u32().ok_or(ProgramError::ParsingError)?;

            let res = runtime.get_current_context()?.declare_const(num_val);
            // Add const to circuit only if the declaration was successful
            if res.is_ok() {
                // Setting as id the constant value
                res?;
                ac.add_const_var(num_val, num_val);
            }

            Ok(num_val.to_string())
        }
        Expression::InfixOp {
            lhe, infix_op, rhe, ..
        } => {
            let varlop = traverse_expression(ac, runtime, lhe, _program_archive)?;
            let varrop = traverse_expression(ac, runtime, rhe, _program_archive)?;

            traverse_infix_op(ac, runtime, &varlop, &varrop, infix_op)
        }
        Expression::Variable { name, access, .. } => {
            let mut name_access = String::from(name);
            debug!("Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let dim_u32_str = traverse_expression(ac, runtime, expr, _program_archive)?;
                        name_access.push('_');
                        name_access.push_str(&dim_u32_str);
                        debug!("Changed var name to {}", name_access);
                    }
                    Access::ComponentAccess(_) => {
                        debug!("Component access found");
                        todo!()
                    }
                }
            }

            Ok(name_access)
        }
        Expression::Call { id, args, .. } => {
            debug!("Call found {}", id);

            // We always need to distinguish a function call from a template component wiring
            let functions = _program_archive.get_function_names();
            let arg_names = if functions.contains(id) {
                _program_archive.get_function_data(id).get_name_of_params()
            } else {
                _program_archive.get_template_data(id).get_name_of_params()
            };

            let mut args_map = HashMap::new();

            // We start by setting argument values to argument names
            for (_arg_name, arg_value) in arg_names.iter().zip(args) {
                // We set arg_name to have arg_value
                // Because arg_value is an expression (constant, variable, or an infix operation or a function call) we need to execute to have the actual value
                let value = execute_expression(ac, runtime, arg_value, _program_archive)?;
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
                _program_archive.get_function_data(id).get_body_as_vec()
            } else {
                _program_archive.get_template_data(id).get_body_as_vec()
            };
            
            traverse_sequence_of_statements(ac, runtime, _body, _program_archive, true)?;

            Ok(id.to_string())
        }
        _ => unimplemented!("Expression not implemented"),
    }
}

/// Traverses an infix operation and processes it based on the data types of the inputs.
/// - If both inputs are scalar variables, it directly computes the operation.
/// - If one or both inputs are not scalar variables, it constructs the corresponding circuit gate.
/// Returns a variable containing the result of the operation or the signal of the output gate.
pub fn traverse_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    input_lhs: &str,
    input_rhs: &str,
    infixop: &ExpressionInfixOpcode,
) -> Result<String, ProgramError> {
    let ctx = runtime.get_current_context()?;

    // Retrieve left and right hand side data items
    let lhs_data = ctx.get_data_item(input_lhs)?;
    let rhs_data = ctx.get_data_item(input_rhs)?;

    // Get numerical values of lhs and rhs
    let lhs_value = lhs_data.get_u32()?;
    let rhs_value = rhs_data.get_u32()?;

    let lhs_data_type = lhs_data.get_data_type();
    let rhs_data_type = rhs_data.get_data_type();

    // Execute directly for scalar values
    if lhs_data_type == DataType::Variable && rhs_data_type == DataType::Variable {
        let result_var = ctx.declare_auto_var()?;
        ctx.set_data_item(
            &result_var,
            DataContent::Scalar(execute_infix_op(&lhs_value, &rhs_value, infixop)),
        )?;
        return Ok(result_var);
    }

    // Add constants to the circuit if needed
    if lhs_data_type == DataType::Variable {
        ac.add_const_var(lhs_value, lhs_value);
    }
    if rhs_data_type == DataType::Variable {
        ac.add_const_var(rhs_value, rhs_value);
    }

    // Create and add a new gate for non-scalar operations
    let gate_type = AGateType::from(infixop);
    let output_signal = ctx.declare_auto_signal()?;
    let output_id = ctx.get_data_item(&output_signal)?.get_u32()?;

    ac.add_gate(&output_signal, output_id, lhs_value, rhs_value, gate_type);

    Ok(output_signal)
}

/// Handles declaration of signals and variables
pub fn traverse_declaration(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    var_name: &str,
    xtype: &VariableType,
    dim_u32_vec: &[u32],
) -> Result<(), ProgramError> {
    let ctx = runtime.get_current_context()?;
    let is_array = !dim_u32_vec.is_empty();

    match xtype {
        VariableType::Signal(_, _) => {
            if is_array {
                for &i in dim_u32_vec {
                    let (name, id) = ctx.declare_signal_array(var_name, vec![i])?;
                    ac.add_var(id, &name);
                }
            } else {
                let signal_id = ctx.declare_signal(var_name)?;
                ac.add_var(signal_id, var_name);
            }
        }
        VariableType::Var => {
            if is_array {
                for &i in dim_u32_vec {
                    ctx.declare_var_array(var_name, vec![i])?;
                }
            } else {
                ctx.declare_variable(var_name)?;
            }
        }
        _ => unimplemented!(),
    }

    Ok(())
}
