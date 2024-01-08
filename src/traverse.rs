//! # Traverse Module
//!
//! This module provides functionality for traversing statements, expressions, infix operations and declaration of components, signals and variables.
//!
//! It's main purpose is to traverse signals.

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::execute::{execute_expression, execute_infix_op, execute_statement};
use crate::program::ProgramError;
use crate::runtime::{DataType, Runtime};
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
        }
        Statement::Declaration {
            xtype,
            name,
            dimensions,
            ..
        } => {
            debug!("Declaration of {}", name);

            // Process index in case of array
            let dim_u32_vec: Vec<u32> = dimensions
                .iter()
                .map(|dimension| execute_expression(ac, runtime, name, dimension, program_archive))
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
        Statement::While { cond, stmt, .. } => loop {
            let var = String::from("while");

            let result = execute_expression(ac, runtime, &var, cond, program_archive)?;
            if result == 0 {
                break;
            }

            debug!("While res = {}", result);
            traverse_statement(ac, runtime, stmt, program_archive)?
        },
        Statement::IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            let var = String::from("IFTHENELSE");
            let result = execute_expression(ac, runtime, &var, cond, program_archive)?;
            let else_case = else_case.as_ref().map(|e| e.as_ref());
            if result == 0 {
                if let Option::Some(else_stmt) = else_case {
                    traverse_statement(ac, runtime, else_stmt, program_archive)?;
                }
            } else {
                traverse_statement(ac, runtime, if_case, program_archive)?
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
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive)?;
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
                        traverse_expression(ac, runtime, &name_access, rhe, program_archive)?;
                    }
                    DataType::Variable => {
                        execute_statement(ac, runtime, stmt, program_archive)?;
                    }
                }
            }
        }
        Statement::Block { stmts, .. } => {
            traverse_sequence_of_statements(ac, runtime, stmts, program_archive, true)?;
        }
        _ => unimplemented!("Statement not implemented"),
    }

    Ok(())
}

/// Examines an expression to determine its structure and dependencies before execution.
pub fn traverse_expression(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    var: &String,
    expression: &Expression,
    _program_archive: &ProgramArchive,
) -> Result<String, ProgramError> {
    match expression {
        Expression::Number(_, value) => {
            // Declaring a constant.
            let val = value.to_u32().ok_or(ProgramError::ParsingError)?;
            debug!("Number value {}", val);

            let res = runtime.get_current_context()?.declare_const(val);
            // Add const to circuit only if the declaration was successful
            if res.is_ok() {
                // Setting as id the constant value
                ac.add_const_var(val, val);
            }

            Ok(val.to_string())
        }
        Expression::InfixOp {
            lhe, infix_op, rhe, ..
        } => {
            let ctx = runtime.get_current_context()?;

            //TODO: for generic handling we should generate a name for an intermediate expression, we could ideally use only the values returned
            let varlhs = ctx.declare_auto_var()?;
            let varrhs = ctx.declare_auto_var()?;

            let varlop = traverse_expression(ac, runtime, &varlhs, lhe, _program_archive)?;
            let varrop = traverse_expression(ac, runtime, &varrhs, rhe, _program_archive)?;

            let res = traverse_infix_op(ac, runtime, var, &varlop, &varrop, infix_op)?;

            match res {
                Some(value) => Ok(value.to_string()),
                None => Ok(var.to_string()),
            }
        }
        Expression::Variable { name, access, .. } => {
            let mut name_access = String::from(name);
            debug!("Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, _program_archive)?;
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
            println!("Call found {}", id);

            // HERE IS CODE FOR ARGUMENTS

            let functions = _program_archive.get_function_names();
            let arg_names = if functions.contains(id) {
                _program_archive.get_function_data(id).get_name_of_params()
            } else {
                _program_archive.get_template_data(id).get_name_of_params()
            };

            for (arg_name, arg_value) in arg_names.iter().zip(args) {
                // We set arg_name to have arg_value
                execute_expression(ac, runtime, arg_name, arg_value, _program_archive)?;
                // TODO: set res to arg_name
            }

            // HERE IS CODE FOR FUNCTIGON

            let fn_body = _program_archive.get_function_data(id).get_body_as_vec();
            traverse_sequence_of_statements(ac, runtime, fn_body, _program_archive, true)?;

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
            Ok(id.to_string())
        }
        _ => unimplemented!("Expression not implemented"),
    }
}

/// Traverses an infix operation.
/// If both inputs are scalars it will execute and return the result.
/// Otherwise it creates the corresponding circuit gate.
pub fn traverse_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    output: &str,
    input_lhs: &str,
    input_rhs: &str,
    infixop: &ExpressionInfixOpcode,
) -> Result<Option<u32>, ProgramError> {
    debug!("Traversing infix op");

    let ctx = runtime.get_current_context()?;

    // Check availability of lhs and rhs values
    let lhs = ctx.get_data_item(input_lhs)?;
    let rhs = ctx.get_data_item(input_rhs)?;

    // Get values
    let lhs_val = lhs.get_u32()?;
    let rhs_val = rhs.get_u32()?;

    // If both items are scalars we can directly execute.
    if lhs.get_data_type() == DataType::Variable && rhs.get_data_type() == DataType::Variable {
        return Ok(Some(execute_infix_op(&lhs_val, &rhs_val, infixop)));
    }

    // If they're not we construct the gate.
    // TODO: Fix, this will fail if the output is not assigned/declared
    let output_id = ctx.get_data_item(output)?.get_u32()?;

    let gate_type = AGateType::from(infixop);

    ac.add_gate(output, output_id, lhs_val, rhs_val, gate_type);

    Ok(None)
}

/// Handles declaration of signals and variables
pub fn traverse_declaration(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    var_name: &str,
    xtype: &VariableType,
    dim_u32_vec: &Vec<u32>,
) {
    let ctx = runtime.get_current_context().unwrap();
    let is_array = !dim_u32_vec.is_empty();

    match xtype {
        VariableType::Signal(_, _) => {
            if is_array {
                let dim_u32 = *dim_u32_vec.last().unwrap();
                for i in 0..dim_u32 {
                    let (name, id) = ctx.declare_signal_array(var_name, vec![i]).unwrap();
                    ac.add_var(id, &name);
                }
            } else {
                let signal_id = ctx.declare_signal(var_name).unwrap();
                ac.add_var(signal_id, var_name.to_string().as_str());
            }
        }
        VariableType::Var => {
            if is_array {
                let dim_u32 = *dim_u32_vec.last().unwrap();
                for i in 0..dim_u32 {
                    ctx.declare_var_array(var_name, vec![i]).unwrap();
                }
            } else {
                ctx.declare_variable(var_name).unwrap();
            }
        }
        _ => unimplemented!(),
    }
}
