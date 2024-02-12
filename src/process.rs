//! # Process Module
//!
//! Handles execution of statements and expressions for arithmetic circuit generation within a `Runtime` environment.

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::program::ProgramError;
use crate::runtime::{increment_indices, u32_to_access, DataAccess, DataType, Runtime, SubAccess};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{Access, Expression, ExpressionInfixOpcode, Statement};
use circom_program_structure::program_archive::ProgramArchive;

const RETURN_VAR: &str = "function_return";

/// Processes a sequence of statements.
pub fn process_statements(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    statements: &[Statement],
    program_archive: &ProgramArchive,
    _is_complete_template: bool,
) -> Result<(), ProgramError> {
    for statement in statements {
        process_statement(ac, runtime, statement, program_archive)?;
    }

    Ok(())
}

/// Processes a single statement.
pub fn process_statement(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    stmt: &Statement,
    program_archive: &ProgramArchive,
) -> Result<(), ProgramError> {
    match stmt {
        Statement::Block { stmts, .. } => {
            process_statements(ac, runtime, stmts, program_archive, true)
        }
        Statement::InitializationBlock {
            initializations, ..
        } => {
            for statement in initializations {
                process_statement(ac, runtime, statement, program_archive)?;
            }

            Ok(())
        }
        Statement::Declaration {
            xtype,
            name,
            dimensions,
            ..
        } => {
            let data_type = DataType::try_from(xtype)?;
            let dim_access: Vec<DataAccess> = dimensions
                .iter()
                .map(|exp| process_expression(ac, runtime, exp, program_archive))
                .collect::<Result<Vec<DataAccess>, ProgramError>>()?;

            let ctx = runtime.current_context()?;
            let dimensions: Vec<u32> = dim_access
                .iter()
                .map(|dim_access| {
                    ctx.get_variable(dim_access)?
                        .ok_or(ProgramError::EmptyDataItem)
                })
                .collect::<Result<Vec<u32>, ProgramError>>()?;
            ctx.declare_item(data_type.clone(), name, &dimensions)?;

            // If the declared item is a signal we should add it to the arithmetic circuit
            if data_type == DataType::Signal {
                let mut signal_access = DataAccess::new(name, Vec::new());

                if dimensions.is_empty() {
                    let signal_id = ctx.get_signal(&signal_access)?;
                    ac.add_var(signal_id, &signal_id.to_string());
                } else {
                    let mut indices: Vec<u32> = vec![0; dimensions.len()];

                    loop {
                        // Set access and get signal id for the current indices
                        signal_access.set_access(u32_to_access(&indices));
                        let signal_id = ctx.get_signal(&signal_access)?;
                        ac.add_var(signal_id, &signal_id.to_string());

                        // Increment indices
                        if !increment_indices(&mut indices, &dimensions)? {
                            break;
                        }
                    }
                }
            }

            Ok(())
        }
        Statement::While { cond, stmt, .. } => {
            loop {
                let access = process_expression(ac, runtime, cond, program_archive)?;
                let result = runtime
                    .current_context()?
                    .get_variable(&access)?
                    .ok_or(ProgramError::EmptyDataItem)?;

                if result == 0 {
                    break;
                }

                process_statement(ac, runtime, stmt, program_archive)?;
            }

            Ok(())
        }
        Statement::IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            let access = process_expression(ac, runtime, cond, program_archive)?;
            let result = runtime
                .current_context()?
                .get_variable(&access)?
                .ok_or(ProgramError::EmptyDataItem)?;

            if result == 0 {
                if let Some(else_stmt) = else_case {
                    process_statement(ac, runtime, else_stmt, program_archive)
                } else {
                    Ok(())
                }
            } else {
                process_statement(ac, runtime, if_case, program_archive)
            }
        }
        Statement::Substitution {
            var, access, rhe, ..
        } => {
            let data_type = {
                let ctx = runtime.current_context()?;
                ctx.get_item_data_type(var)?
            };

            let access = build_access(runtime, ac, program_archive, var, access)?;

            match data_type {
                DataType::Signal => {
                    // This corresponds to a gate generation
                    let temp_output = process_expression(ac, runtime, rhe, program_archive)?;

                    // Replace the temporary output signal with the given one.
                    let ctx = runtime.current_context()?;
                    let temp_output_id = ctx.get_signal(&temp_output)?;
                    let given_output_id = ctx.get_signal(&access)?;

                    ac.replace_output_var_in_gate(temp_output_id, given_output_id);
                }
                DataType::Variable => {
                    // This corresponds to a variable assignment
                    let value_access = process_expression(ac, runtime, rhe, program_archive)?;
                    let value = runtime.current_context()?.get_variable(&value_access)?;
                    let ctx = runtime.current_context()?;
                    ctx.set_variable(&access, value)?;
                }
                DataType::Component => {
                    // This corresponds to a component wiring
                    let rhs = process_expression(ac, runtime, rhe, program_archive)?;

                    // Add connection
                    let ctx = runtime.current_context()?;
                    ctx.add_connection(var, access, rhs)?;
                }
            }

            Ok(())
        }
        Statement::Return { value, .. } => {
            let return_var_access = DataAccess::new(RETURN_VAR, vec![]);
            let return_access = process_expression(ac, runtime, value, program_archive)?;
            let return_value = runtime
                .current_context()?
                .get_variable(&return_access)?
                .ok_or(ProgramError::EmptyDataItem)?;

            let ctx = runtime.current_context()?;
            if ctx.get_variable(&return_var_access).is_err() {
                ctx.declare_item(DataType::Variable, RETURN_VAR, &[])?;
            }
            ctx.set_variable(&return_var_access, Some(return_value))?;

            Ok(())
        }
        _ => unimplemented!("Statement processing not implemented"),
    }
}

/// Processes an expression and returns an access to the result.
pub fn process_expression(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    expression: &Expression,
    program_archive: &ProgramArchive,
) -> Result<DataAccess, ProgramError> {
    match expression {
        Expression::InfixOp {
            lhe, infix_op, rhe, ..
        } => {
            let varlop = process_expression(ac, runtime, lhe, program_archive)?;
            let varrop = process_expression(ac, runtime, rhe, program_archive)?;

            traverse_infix_op(ac, runtime, &varlop, &varrop, infix_op)
        }
        Expression::Call { id, args, .. } => {
            // Determine if the call is to a function or a template and get argument names and body
            let (arg_names, body) = if program_archive.contains_function(id) {
                let function_data = program_archive.get_function_data(id);
                (
                    function_data.get_name_of_params().clone(),
                    function_data.get_body_as_vec().to_vec(),
                )
            } else if program_archive.contains_template(id) {
                let template_data = program_archive.get_template_data(id);
                (
                    template_data.get_name_of_params().clone(),
                    template_data.get_body_as_vec().to_vec(),
                )
            } else {
                return Err(ProgramError::UndefinedFunctionOrTemplate);
            };

            let arg_values = args
                .iter()
                .map(|arg_expr| {
                    process_expression(ac, runtime, arg_expr, program_archive).and_then(
                        |value_access| {
                            runtime
                                .current_context()?
                                .get_variable(&value_access)?
                                .ok_or(ProgramError::EmptyDataItem)
                        },
                    )
                })
                .collect::<Result<Vec<u32>, ProgramError>>()?;

            // Create a new context for the function/template execution
            runtime.push_context(false)?;

            // Scope for the new context operations
            {
                let ctx = runtime.current_context()?;

                // Declare and set argument variables in the new context
                for (arg_name, &arg_value) in arg_names.iter().zip(&arg_values) {
                    ctx.declare_item(DataType::Variable, arg_name, &[])?;
                    ctx.set_variable(&DataAccess::new(arg_name, vec![]), Some(arg_value))?;
                }

                // Process the function/template body
                process_statements(ac, runtime, &body, program_archive, true)?;
            }

            // Retrieve the return value before and pop the context
            let mut return_value: Option<u32> = None;
            if let Ok(value) = runtime
                .current_context()?
                .get_variable(&DataAccess::new(RETURN_VAR, vec![]))
            {
                return_value = value;
            }
            runtime.pop_context(false)?;

            // Store the return value in the parent context
            let return_access = DataAccess::new(&format!("{}_{}", id, RETURN_VAR), vec![]);
            runtime.current_context()?.declare_item(
                DataType::Variable,
                &return_access.get_name(),
                &[],
            )?;
            runtime
                .current_context()?
                .set_variable(&return_access, return_value)?;

            Ok(return_access)
        }
        Expression::Number(_, value) => {
            let ctx = runtime.current_context()?;
            let access = ctx.declare_random_item(DataType::Variable)?;

            ctx.set_variable(
                &access,
                Some(value.to_u32().ok_or(ProgramError::ParsingError)?),
            )?;

            Ok(access)
        }
        Expression::Variable { name, access, .. } => {
            build_access(runtime, ac, program_archive, name, access)
        }
        _ => unimplemented!("Expression not implemented"),
    }
}

/// Traverses an infix operation and processes it based on the data types of the inputs.
/// - If both inputs are variables, it directly computes the operation.
/// - If one or both inputs are signals, it constructs the corresponding circuit gate.
/// Returns the access to a variable containing the result of the operation or the signal of the output gate.
pub fn traverse_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    input_lhs: &DataAccess,
    input_rhs: &DataAccess,
    op: &ExpressionInfixOpcode,
) -> Result<DataAccess, ProgramError> {
    let ctx = runtime.current_context()?;

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

        let op_res = execute_op(&lhs_value, &rhs_value, op);
        let item_access = ctx.declare_random_item(DataType::Variable)?;
        ctx.set_variable(&item_access, Some(op_res))?;

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
    let gate_type = AGateType::from(op);
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
                let index_access = process_expression(ac, runtime, expr, program_archive)?;
                let index = runtime
                    .current_context()?
                    .get_variable(&index_access)?
                    .ok_or(ProgramError::EmptyDataItem)?;
                access_vec.push(SubAccess::Array(index));
            }
            Access::ComponentAccess(signal) => {
                access_vec.push(SubAccess::Component(signal.to_string()));
            }
        }
    }

    Ok(DataAccess::new(name, access_vec))
}

/// Executes an operation, performing the specified arithmetic or logical computation.
pub fn execute_op(lhs: &u32, rhs: &u32, op: &ExpressionInfixOpcode) -> u32 {
    match AGateType::from(op) {
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
