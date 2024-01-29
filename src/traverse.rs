//! # Traverse Module
//!
//! This module provides functionality for traversing statements, expressions, infix operations and declaration of components, signals and variables.
//!
//! It's main purpose is to traverse signals.

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::execute::{execute_expression, execute_infix_op};
use crate::program::ProgramError;
use crate::runtime::{ContextOrigin, DataAccess, DataType, Runtime, SubAccess};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{Access, Expression, ExpressionInfixOpcode, Statement};
use circom_program_structure::program_archive::ProgramArchive;
use log::debug;
use std::collections::HashMap;

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
            let dim: Vec<u32> = dimensions
                .iter()
                .map(|exp| {
                    execute_expression(ac, runtime, exp, program_archive)?
                        .ok_or(ProgramError::EmptyDataItem)
                })
                .collect::<Result<Vec<u32>, _>>()?;
            let ctx = runtime.get_current_context()?;

            ctx.declare_item(DataType::try_from(xtype)?, name, &dim)?;

            Ok(())
        }
        Statement::While { cond, stmt, .. } => {
            loop {
                let result = execute_expression(ac, runtime, cond, program_archive)?;
                if result == Some(0) {
                    break;
                }

                debug!("While res = {:?}", result);
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
            if result == Some(0) {
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

            // Evaluate the data type of the given variable
            let ctx = runtime.get_current_context()?;
            let data_type = ctx.get_item_data_type(var)?;

            // Build access
            let _access = build_access(runtime, ac, program_archive, var, access)?;

            match data_type {
                DataType::Signal => todo!(),
                DataType::Variable => todo!(),
                DataType::Component => {
                    // Process right hand expression
                    let _rhs = traverse_expression(ac, runtime, rhe, program_archive)?;

                    // Add connection
                }
            }

            // Check if we're dealing with a signal or a variable
            // let ctx = runtime.get_current_context()?;
            // let data_item = ctx.get_data_item(&name_access);
            // if let Ok(data_value) = data_item {
            //     match data_value.get_data_type() {
            //         DataType::Signal => {
            //             traverse_expression(ac, runtime, rhe, program_archive)?;
            //         }
            //         DataType::Variable => {
            //             execute_statement(ac, runtime, stmt, program_archive)?;
            //         }
            //         DataType::Component => {
            //             //Here we deal with wiring
            //             //lhs is a component wire and rhs is a signal

            //             // we also check to complete template if all wiring is done
            //         }
            //     }
            // }

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
        _ => unimplemented!("Statement not implemented"),
    }
}

/// Process an expression and returns a name of a data item that contains the result.
pub fn traverse_expression(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    expression: &Expression,
    _program_archive: &ProgramArchive,
) -> Result<DataAccess, ProgramError> {
    match expression {
        Expression::Number(_, value) => {
            let ctx = runtime.get_current_context()?;
            let int = value.to_u32().ok_or(ProgramError::ParsingError)?;
            let access = DataAccess::new(int.to_string(), vec![]);

            let res = ctx.declare_item(DataType::Variable, &access.get_name(), &[]);

            // Add const to circuit only if the declaration was successful
            if res.is_ok() {
                res?;
                ac.add_const_var(int, int);
            }

            Ok(access)
        }
        Expression::InfixOp {
            lhe, infix_op, rhe, ..
        } => {
            let varlop = traverse_expression(ac, runtime, lhe, _program_archive)?;
            let varrop = traverse_expression(ac, runtime, rhe, _program_archive)?;

            traverse_infix_op(ac, runtime, &varlop, &varrop, infix_op)
        }
        Expression::Variable { name, access, .. } => {
            build_access(runtime, ac, _program_archive, name, access)
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

            let mut args_map: HashMap<String, u32> = HashMap::new();

            // We start by setting argument values to argument names
            for (arg_name, arg_value) in arg_names.iter().zip(args) {
                // We set arg_name to have arg_value
                // Because arg_value is an expression (constant, variable, or an infix operation or a function call) we need to execute to have the actual value
                let value = execute_expression(ac, runtime, arg_value, _program_archive)?
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
                _program_archive.get_function_data(id).get_body_as_vec()
            } else {
                _program_archive.get_template_data(id).get_body_as_vec()
            };

            traverse_sequence_of_statements(ac, runtime, _body, _program_archive, true)?;

            if functions.contains(id) {
                // let ret = ctx.get_data_item("RETURN").unwrap().get_u32().unwrap();
                // runtime.pop_context();
                Ok(DataAccess::new(id.to_string(), vec![]))
            } else {
                // runtime.pop_context();
                Ok(DataAccess::new(id.to_string(), vec![]))
            }
        }
        _ => unimplemented!("Expression not implemented"),
    }
}

/// Traverses an infix operation and processes it based on the data types of the inputs.
/// - If both inputs are variables, it directly computes the operation.
/// - If one or both inputs are signals, it constructs the corresponding circuit gate.
/// Returns a variable containing the result of the operation or the signal of the output gate.
pub fn traverse_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    input_lhs: &DataAccess,
    input_rhs: &DataAccess,
    infixop: &ExpressionInfixOpcode,
) -> Result<DataAccess, ProgramError> {
    let ctx = runtime.get_current_context()?;

    // Determine the data types of the left and right operands
    let lhs_data_type = ctx.get_item_data_type(&input_lhs.get_name())?;
    let rhs_data_type = ctx.get_item_data_type(&input_rhs.get_name())?;

    // Handle the case where both inputs are variables
    if lhs_data_type == DataType::Variable && rhs_data_type == DataType::Variable {
        let lhs_value = ctx
            .get_variable(input_lhs)?
            .ok_or(ProgramError::EmptyDataItem)?;
        let rhs_value = ctx
            .get_variable(input_rhs)?
            .ok_or(ProgramError::EmptyDataItem)?;

        let op_res = execute_infix_op(&lhs_value, &rhs_value, infixop);
        let item_access = ctx.declare_random_item(DataType::Variable)?;
        ctx.set_variable(item_access.clone(), Some(op_res))?;

        return Ok(item_access);
    }

    // Handle cases where one or both inputs are signals
    let lhs_signal = match lhs_data_type {
        DataType::Signal => ctx.get_signal(input_lhs)?,
        DataType::Variable => {
            let value = ctx
                .get_variable(input_lhs)?
                .ok_or(ProgramError::EmptyDataItem)?;
            ac.add_const_var(value, value);
            value
        }
        _ => return Err(ProgramError::InvalidDataType),
    };

    let rhs_signal = match rhs_data_type {
        DataType::Signal => ctx.get_signal(input_rhs)?,
        DataType::Variable => {
            let value = ctx
                .get_variable(input_rhs)?
                .ok_or(ProgramError::EmptyDataItem)?;
            ac.add_const_var(value, value);
            value
        }
        _ => return Err(ProgramError::InvalidDataType),
    };

    // Construct the corresponding circuit gate
    let gate_type = AGateType::from(infixop);
    let output_signal = ctx.declare_random_item(DataType::Signal)?;
    let output_id = ctx.get_signal(&output_signal)?;
    ac.add_gate(
        &output_signal.get_name(),
        output_id,
        lhs_signal,
        rhs_signal,
        gate_type,
    );

    Ok(output_signal)
}

/// Builds a DataAccess from an Access array
pub fn build_access(
    runtime: &mut Runtime,
    ac: &mut ArithmeticCircuit,
    program_archive: &ProgramArchive,
    name: &str,
    access: &[Access],
) -> Result<DataAccess, ProgramError> {
    let mut access_vec = Vec::new();

    for a in access.iter() {
        match a {
            Access::ArrayAccess(expr) => {
                let index = execute_expression(ac, runtime, expr, program_archive)?
                    .ok_or(ProgramError::EmptyDataItem)?;
                access_vec.push(SubAccess::Array(index));
            }
            Access::ComponentAccess(signal) => {
                access_vec.push(SubAccess::Component(signal.to_string()));
            }
        }
    }

    Ok(DataAccess::new(name.to_string(), access_vec))
}
