//! # Process Module
//!
//! Handles execution of statements and expressions for arithmetic circuit generation within a `Runtime` environment.

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::program::ProgramError;
use crate::runtime::{
    increment_indices, u32_to_access, Context, DataAccess, DataType, Runtime, Signal, SubAccess,
};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{
    Access, AssignOp, Expression, ExpressionInfixOpcode, Statement,
};
use circom_program_structure::program_archive::ProgramArchive;
use log::debug;
use std::collections::HashMap;

pub const RETURN_VAR: &str = "function_return";

/// Processes a sequence of statements.
pub fn process_statements(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    program_archive: &ProgramArchive,
    statements: &[Statement],
) -> Result<(), ProgramError> {
    for statement in statements {
        process_statement(ac, runtime, program_archive, statement)?;
    }

    Ok(())
}

/// Processes a single statement.
pub fn process_statement(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    program_archive: &ProgramArchive,
    statement: &Statement,
) -> Result<(), ProgramError> {
    match statement {
        Statement::Block { stmts, .. } => process_statements(ac, runtime, program_archive, stmts),
        Statement::InitializationBlock {
            initializations, ..
        } => {
            for stmt in initializations {
                process_statement(ac, runtime, program_archive, stmt)?;
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
                .map(|expression| process_expression(ac, runtime, program_archive, expression))
                .collect::<Result<Vec<DataAccess>, ProgramError>>()?;

            let ctx = runtime.current_context()?;
            let dimensions: Vec<u32> = dim_access
                .iter()
                .map(|dim_access| {
                    ctx.get_variable_value(dim_access)?
                        .ok_or(ProgramError::EmptyDataItem)
                })
                .collect::<Result<Vec<u32>, ProgramError>>()?;
            ctx.declare_item(data_type.clone(), name, &dimensions)?;

            // If the declared item is a signal we should add it to the arithmetic circuit
            if data_type == DataType::Signal {
                let mut signal_access = DataAccess::new(name, Vec::new());

                if dimensions.is_empty() {
                    let signal_id = ctx.get_signal_id(&signal_access)?;
                    ac.add_var(signal_id, &signal_id.to_string());
                } else {
                    let mut indices: Vec<u32> = vec![0; dimensions.len()];

                    loop {
                        // Set access and get signal id for the current indices
                        signal_access.set_access(u32_to_access(&indices));
                        let signal_id = ctx.get_signal_id(&signal_access)?;
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
            // TODO: instantiate a new context
            loop {
                let access = process_expression(ac, runtime, program_archive, cond)?;
                let result = runtime
                    .current_context()?
                    .get_variable_value(&access)?
                    .ok_or(ProgramError::EmptyDataItem)?;

                if result == 0 {
                    break;
                }

                process_statement(ac, runtime, program_archive, stmt)?;
            }

            Ok(())
        }
        Statement::IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            // TODO: instantiate a new context
            let access = process_expression(ac, runtime, program_archive, cond)?;
            let result = runtime
                .current_context()?
                .get_variable_value(&access)?
                .ok_or(ProgramError::EmptyDataItem)?;

            if result == 0 {
                if let Some(else_statement) = else_case {
                    process_statement(ac, runtime, program_archive, else_statement)
                } else {
                    Ok(())
                }
            } else {
                process_statement(ac, runtime, program_archive, if_case)
            }
        }
        Statement::Substitution {
            var,
            access,
            rhe,
            op,
            ..
        } => {
            let data_type = runtime.current_context()?.get_item_data_type(var)?;
            let lh_access = build_access(ac, runtime, program_archive, var, access)?;
            let rh_access = process_expression(ac, runtime, program_archive, rhe)?;

            let ctx = runtime.current_context()?;
            match data_type {
                DataType::Signal => {
                    // Create a temporary signal and replace the output variable in the gate
                    let temp_output_id = ctx.get_signal_id(&rh_access)?;
                    let given_output_id = ctx.get_signal_id(&lh_access)?;
                    ac.replace_output_var_in_gate(temp_output_id, given_output_id);
                }
                DataType::Variable => {
                    // Assign the evaluated right-hand side to the left-hand side
                    let value = ctx.get_variable_value(&rh_access)?;
                    ctx.set_variable(&lh_access, value)?;
                }
                DataType::Component => match op {
                    AssignOp::AssignVar => {
                        // Component assignment
                        let signal_map = ctx.get_component_map(&rh_access)?;
                        ctx.set_component(&lh_access, signal_map)?;
                    }
                    AssignOp::AssignConstraintSignal => {
                        // Wiring
                        // Get the component's signal id (old id)
                        let old_id = ctx.get_component_signal_id(&lh_access)?;

                        // Get the assigned signal id (new id)
                        let new_id = get_signal_for_access(ac, &ctx, &rh_access)?;

                        debug!("Wiring old signal {} to new signal {}", old_id, new_id);

                        // Replace id in the circuit
                        ac.replace_var_id(old_id, new_id)
                    }
                    _ => return Err(ProgramError::ParsingError),
                },
            }

            Ok(())
        }
        Statement::Return { value, .. } => {
            let return_var_access = DataAccess::new(RETURN_VAR, vec![]);
            let return_access = process_expression(ac, runtime, program_archive, value)?;
            let return_value = runtime
                .current_context()?
                .get_variable_value(&return_access)?
                .ok_or(ProgramError::EmptyDataItem)?;

            let ctx = runtime.current_context()?;
            if ctx.get_variable_value(&return_var_access).is_err() {
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
    program_archive: &ProgramArchive,
    expression: &Expression,
) -> Result<DataAccess, ProgramError> {
    match expression {
        Expression::Call { id, args, .. } => handle_call(ac, runtime, program_archive, id, args),
        Expression::InfixOp {
            lhe, infix_op, rhe, ..
        } => handle_infix_op(ac, runtime, program_archive, infix_op, lhe, rhe),
        Expression::Number(_, value) => {
            let access = runtime
                .current_context()?
                .declare_random_item(DataType::Variable)?;

            runtime.current_context()?.set_variable(
                &access,
                Some(value.to_u32().ok_or(ProgramError::ParsingError)?),
            )?;

            Ok(access)
        }
        Expression::Variable { name, access, .. } => {
            build_access(ac, runtime, program_archive, name, access)
        }
        _ => unimplemented!("Expression not implemented"),
    }
}

/// Handles function and template calls.
fn handle_call(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    program_archive: &ProgramArchive,
    id: &str,
    args: &[Expression],
) -> Result<DataAccess, ProgramError> {
    // Determine if the call is to a function or a template and get argument names and body
    let is_function = program_archive.contains_function(id);
    let (arg_names, body) = if is_function {
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
            process_expression(ac, runtime, program_archive, arg_expr).and_then(|value_access| {
                runtime
                    .current_context()?
                    .get_variable_value(&value_access)?
                    .ok_or(ProgramError::EmptyDataItem)
            })
        })
        .collect::<Result<Vec<u32>, ProgramError>>()?;

    // Create a new execution context
    runtime.push_context(false)?;

    // Set arguments in the new context
    for (arg_name, &arg_value) in arg_names.iter().zip(&arg_values) {
        runtime
            .current_context()?
            .declare_item(DataType::Variable, arg_name, &[])?;
        runtime
            .current_context()?
            .set_variable(&DataAccess::new(arg_name, vec![]), Some(arg_value))?;
    }

    // Process the function/template body
    process_statements(ac, runtime, program_archive, &body)?;

    // Get return values
    let mut function_return: Option<u32> = None;
    let mut component_return: HashMap<String, Signal> = HashMap::new();

    if is_function {
        if let Ok(value) = runtime
            .current_context()?
            .get_variable_value(&DataAccess::new(RETURN_VAR, vec![]))
        {
            function_return = value;
        }
    } else {
        // Retrieve input and output signals
        let template_data = program_archive.get_template_data(id);
        let input_signals = template_data.get_inputs();
        let output_signals = template_data.get_outputs();

        // Store ids in the component
        for (signal, _) in input_signals.iter().chain(output_signals.iter()) {
            let ids = runtime.current_context()?.get_signal(signal)?;
            component_return.insert(signal.to_string(), ids);
        }
    }

    // Return to parent context
    runtime.pop_context(false)?;
    let ctx = runtime.current_context()?;
    let return_access = DataAccess::new(&format!("{}_{}", id, RETURN_VAR), vec![]);

    if is_function {
        ctx.declare_item(DataType::Variable, &return_access.get_name(), &[])?;
        ctx.set_variable(&return_access, function_return)?;
    } else {
        ctx.declare_item(DataType::Component, &return_access.get_name(), &[])?;
        ctx.set_component(&return_access, component_return)?;
    }

    Ok(return_access)
}

/// Handles an infix operation.
/// - If both inputs are variables, it directly computes the operation.
/// - If one or both inputs are signals, it constructs the corresponding circuit gate.
/// Returns the access to a variable containing the result of the operation or the signal of the output gate.
fn handle_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    program_archive: &ProgramArchive,
    op: &ExpressionInfixOpcode,
    lhe: &Expression,
    rhe: &Expression,
) -> Result<DataAccess, ProgramError> {
    let lhe_access = process_expression(ac, runtime, program_archive, lhe)?;
    let rhe_access = process_expression(ac, runtime, program_archive, rhe)?;

    let ctx = runtime.current_context()?;

    // Determine the data types of the left and right operands
    let lhs_data_type = ctx.get_item_data_type(&lhe_access.get_name())?;
    let rhs_data_type = ctx.get_item_data_type(&rhe_access.get_name())?;

    // Handle the case where both inputs are variables
    if lhs_data_type == DataType::Variable && rhs_data_type == DataType::Variable {
        let lhs_value = ctx
            .get_variable_value(&lhe_access)?
            .ok_or(ProgramError::EmptyDataItem)?;
        let rhs_value = ctx
            .get_variable_value(&rhe_access)?
            .ok_or(ProgramError::EmptyDataItem)?;

        let op_res = execute_op(&lhs_value, &rhs_value, op);
        let item_access = ctx.declare_random_item(DataType::Variable)?;
        ctx.set_variable(&item_access, Some(op_res))?;

        return Ok(item_access);
    }

    // Handle cases where one or both inputs are signals
    let lhs_signal = get_signal_for_access(ac, &ctx, &lhe_access)?;
    let rhs_signal = get_signal_for_access(ac, &ctx, &rhe_access)?;

    // Construct the corresponding circuit gate
    let gate_type = AGateType::from(op);
    let output_signal = ctx.declare_random_item(DataType::Signal)?;
    let output_id = ctx.get_signal_id(&output_signal)?;
    ac.add_gate(
        &output_signal.get_name(),
        output_id,
        lhs_signal,
        rhs_signal,
        gate_type,
    );

    Ok(output_signal)
}

/// Returns a signal id for a given access
/// - If the access is a signal or a component, it returns the corresponding signal id.
/// - If the access is a variable, it adds a constant variable to the circuit and returns the corresponding signal id.
fn get_signal_for_access(
    ac: &mut ArithmeticCircuit,
    ctx: &Context,
    access: &DataAccess,
) -> Result<u32, ProgramError> {
    match ctx.get_item_data_type(&access.get_name())? {
        DataType::Signal => Ok(ctx.get_signal_id(access)?),
        DataType::Variable => {
            let value = ctx
                .get_variable_value(access)?
                .ok_or(ProgramError::EmptyDataItem)?;
            ac.add_const_var(value, value);
            Ok(value)
        }
        DataType::Component => Ok(ctx.get_component_signal_id(access)?),
    }
}

/// Builds a DataAccess from an Access array
pub fn build_access(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    program_archive: &ProgramArchive,
    name: &str,
    access: &[Access],
) -> Result<DataAccess, ProgramError> {
    let mut access_vec = Vec::new();

    for a in access.iter() {
        match a {
            Access::ArrayAccess(expression) => {
                let index_access = process_expression(ac, runtime, program_archive, expression)?;
                let index = runtime
                    .current_context()?
                    .get_variable_value(&index_access)?
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
