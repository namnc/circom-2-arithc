//! # Process Module
//!
//! Handles execution of statements and expressions for arithmetic circuit generation within a `Runtime` environment.

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::program::ProgramError;
use crate::runtime::{
    generate_u32, increment_indices, u32_to_access, Context, DataAccess, DataType, NestedValue,
    Runtime, Signal, SubAccess, RETURN_VAR,
};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{
    Access, AssignOp, Expression, ExpressionInfixOpcode, Statement,
};
use circom_program_structure::program_archive::ProgramArchive;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
        Statement::InitializationBlock {
            initializations, ..
        } => process_statements(ac, runtime, program_archive, initializations),
        Statement::Block { stmts, .. } => process_statements(ac, runtime, program_archive, stmts),
        Statement::Substitution {
            var,
            access,
            rhe,
            op,
            ..
        } => handle_substitution(ac, runtime, program_archive, var, access, rhe, op),
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

            let signal_gen = runtime.get_signal_gen();
            let ctx = runtime.current_context()?;
            let dimensions: Vec<u32> = dim_access
                .iter()
                .map(|dim_access| {
                    ctx.get_variable_value(dim_access)?
                        .ok_or(ProgramError::EmptyDataItem)
                })
                .collect::<Result<Vec<u32>, ProgramError>>()?;
            ctx.declare_item(data_type.clone(), name, &dimensions, signal_gen)?;

            // If the declared item is a signal we should add it to the arithmetic circuit
            if data_type == DataType::Signal {
                let mut signal_access = DataAccess::new(name, Vec::new());

                if dimensions.is_empty() {
                    let signal_id = ctx.get_signal_id(&signal_access)?;
                    ac.add_signal(signal_id, signal_access.access_str(ctx.get_ctx_name()), None)?;
                } else {
                    let mut indices: Vec<u32> = vec![0; dimensions.len()];

                    loop {
                        // Set access and get signal id for the current indices
                        signal_access.set_access(u32_to_access(&indices));
                        let signal_id = ctx.get_signal_id(&signal_access)?;
                        ac.add_signal(signal_id, signal_access.access_str(ctx.get_ctx_name()), None)?;

                        // Increment indices
                        if !increment_indices(&mut indices, &dimensions)? {
                            break;
                        }
                    }
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
            let access = process_expression(ac, runtime, program_archive, cond)?;
            let result = runtime
                .current_context()?
                .get_variable_value(&access)?
                .ok_or(ProgramError::EmptyDataItem)?;

            if result == 0 {
                if let Some(else_statement) = else_case {
                    runtime.push_context(true, "IF_FALSE".to_string())?;
                    process_statement(ac, runtime, program_archive, else_statement)?;
                    runtime.pop_context(true)?;
                    Ok(())
                } else {
                    Ok(())
                }
            } else {
                runtime.push_context(true,  "IF_TRUE".to_string())?;
                process_statement(ac, runtime, program_archive, if_case)?;
                runtime.pop_context(true)?;
                Ok(())
            }
        }
        Statement::While { cond, stmt, .. } => {
            runtime.push_context(true, "WHILE_PRE".to_string())?;
            loop {
                let access = process_expression(ac, runtime, program_archive, cond)?;
                let result = runtime
                    .current_context()?
                    .get_variable_value(&access)?
                    .ok_or(ProgramError::EmptyDataItem)?;

                if result == 0 {
                    break;
                }

                runtime.push_context(true, "WHILE_EXE".to_string())?;
                process_statement(ac, runtime, program_archive, stmt)?;
                runtime.pop_context(true)?;
            }
            runtime.pop_context(true)?;

            Ok(())
        }
        Statement::Return { value, .. } => {
            let return_access = process_expression(ac, runtime, program_archive, value)?;

            let signal_gen = runtime.get_signal_gen();
            let ctx = runtime.current_context()?;
            let return_value = ctx
                .get_variable_value(&return_access)?
                .ok_or(ProgramError::EmptyDataItem)?;

            ctx.declare_item(DataType::Variable, RETURN_VAR, &[], signal_gen)?;
            ctx.set_variable(&DataAccess::new(RETURN_VAR, vec![]), Some(return_value))?;

            Ok(())
        }
        _ => Err(ProgramError::StatementNotImplemented),
    }
}

/// Handles a substitution statement
fn handle_substitution(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    program_archive: &ProgramArchive,
    var: &str,
    access: &[Access],
    rhe: &Expression,
    op: &AssignOp,
) -> Result<(), ProgramError> {
    let lh_access = build_access(ac, runtime, program_archive, var, access)?;
    let rh_access = process_expression(ac, runtime, program_archive, rhe)?;

    let signal_gen = runtime.get_signal_gen();
    let ctx = runtime.current_context()?;
    match ctx.get_item_data_type(var)? {
        DataType::Variable => {
            // Assign the evaluated right-hand side to the left-hand side
            let value = ctx.get_variable_value(&rh_access)?;
            ctx.set_variable(&lh_access, value)?;
        }
        DataType::Component => match op {
            AssignOp::AssignVar => {
                // Component instantiation
                let signal_map = ctx.get_component_map(&rh_access)?;
                ctx.set_component(&lh_access, signal_map)?;
            }
            AssignOp::AssignConstraintSignal => {
                // Component signal assignment
                match ctx.get_component_signal_content(&lh_access)? {
                    NestedValue::Array(signal) => {
                        let assigned_signal_array =
                            match get_signal_content_for_access(ctx, &rh_access)? {
                                NestedValue::Array(array) => array,
                                _ => return Err(ProgramError::InvalidDataType),
                            };

                        connect_signal_arrays(ac, &signal, &assigned_signal_array)?;
                    }
                    NestedValue::Value(_) => {
                        let component_signal = ctx.get_component_signal_id(&lh_access)?;
                        let assigned_signal =
                            get_signal_for_access(ac, ctx, signal_gen, &rh_access)?;

                        ac.add_connection(assigned_signal, component_signal)?;
                    }
                }
            }
            _ => return Err(ProgramError::OperationNotSupported),
        },
        DataType::Signal => {
            match rhe {
                Expression::InfixOp { .. } => {
                    // Construct the corresponding circuit gate for the given operation
                    let given_output_id = ctx.get_signal_id(&lh_access)?;
                    let gate_output_id = get_signal_for_access(ac, ctx, signal_gen, &rh_access)?;

                    // Connect the generated gate output to the actual signal
                    ac.add_connection(gate_output_id, given_output_id)?;
                }
                Expression::Variable { .. } => match ctx.get_signal_content(&lh_access)? {
                    // This corresponds to
                    NestedValue::Array(signal) => {
                        let assigned_signal_array =
                            match get_signal_content_for_access(ctx, &rh_access)? {
                                NestedValue::Array(array) => array,
                                _ => return Err(ProgramError::InvalidDataType),
                            };

                        connect_signal_arrays(ac, &signal, &assigned_signal_array)?;
                    }
                    NestedValue::Value(signal_id) => {
                        let gate_output_id =
                            get_signal_for_access(ac, ctx, signal_gen, &rh_access)?;

                        ac.add_connection(gate_output_id, signal_id)?;
                    }
                },
                _ => {}
            }
        }
    }

    Ok(())
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
            let signal_gen = runtime.get_signal_gen();
            let access = runtime
                .current_context()?
                .declare_random_item(signal_gen, DataType::Variable)?;

            runtime.current_context()?.set_variable(
                &access,
                Some(value.to_u32().ok_or(ProgramError::ParsingError)?),
            )?;

            Ok(access)
        }
        Expression::Variable { name, access, .. } => {
            build_access(ac, runtime, program_archive, name, access)
        }
        _ => Err(ProgramError::ExpressionNotImplemented),
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
    runtime.push_context(false, id.to_string())?;

    // Set arguments in the new context
    for (arg_name, &arg_value) in arg_names.iter().zip(&arg_values) {
        let signal_gen = runtime.get_signal_gen();
        runtime
            .current_context()?
            .declare_item(DataType::Variable, arg_name, &[], signal_gen)?;
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
    let signal_gen = runtime.get_signal_gen();
    let ctx = runtime.current_context()?;
    let return_access =
        DataAccess::new(&format!("{}_{}_{}", id, RETURN_VAR, generate_u32()), vec![]);

    if is_function {
        ctx.declare_item(
            DataType::Variable,
            &return_access.get_name(),
            &[],
            signal_gen,
        )?;
        ctx.set_variable(&return_access, function_return)?;
    } else {
        ctx.declare_item(
            DataType::Component,
            &return_access.get_name(),
            &[],
            signal_gen,
        )?;
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

    let signal_gen: Rc<RefCell<u32>> = runtime.get_signal_gen();
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

        let op_res = execute_op(lhs_value, rhs_value, op)?;
        let item_access = ctx.declare_random_item(signal_gen, DataType::Variable)?;
        ctx.set_variable(&item_access, Some(op_res))?;

        return Ok(item_access);
    }

    // Handle cases where one or both inputs are signals
    let lhs_id = get_signal_for_access(ac, ctx, signal_gen.clone(), &lhe_access)?;
    let rhs_id = get_signal_for_access(ac, ctx, signal_gen.clone(), &rhe_access)?;

    // Construct the corresponding circuit gate
    let gate_type = AGateType::from(op);
    let output_signal = ctx.declare_random_item(signal_gen, DataType::Signal)?;
    let output_id = ctx.get_signal_id(&output_signal)?;

    // Add output signal and gate to the circuit
    ac.add_signal(output_id, output_signal.access_str(ctx.get_ctx_name()), None)?;
    ac.add_gate(gate_type, lhs_id, rhs_id, output_id)?;

    Ok(output_signal)
}

/// Returns a signal id for a given access
/// - If the access is a signal or a component, it returns the corresponding signal id.
/// - If the access is a variable, it adds a constant variable to the circuit and returns the corresponding signal id.
fn get_signal_for_access(
    ac: &mut ArithmeticCircuit,
    ctx: &mut Context,
    signal_gen: Rc<RefCell<u32>>,
    access: &DataAccess,
) -> Result<u32, ProgramError> {
    match ctx.get_item_data_type(&access.get_name())? {
        DataType::Signal => Ok(ctx.get_signal_id(access)?),
        DataType::Variable => {
            // Get variable value
            let value = ctx
                .get_variable_value(access)?
                .ok_or(ProgramError::EmptyDataItem)?;

            let signal_access = DataAccess::new(&format!("const_signal_{}", value), vec![]);
            // Try to get signal id if it exists
            if let Ok(id) = ctx.get_signal_id(&signal_access) {
                Ok(id)
            } else {
                // If it doesn't exist, declare it and add it to the circuit
                ctx.declare_item(DataType::Signal, &signal_access.get_name(), &[], signal_gen)?;
                let signal_id = ctx.get_signal_id(&signal_access)?;
                ac.add_signal(signal_id, signal_access.access_str(ctx.get_ctx_name()), Some(value))?;
                Ok(signal_id)
            }
        }
        DataType::Component => Ok(ctx.get_component_signal_id(access)?),
    }
}

/// Returns the content of a signal for a given access
fn get_signal_content_for_access(
    ctx: &Context,
    access: &DataAccess,
) -> Result<NestedValue<u32>, ProgramError> {
    match ctx.get_item_data_type(&access.get_name())? {
        DataType::Signal => Ok(ctx.get_signal_content(access)?),
        DataType::Component => Ok(ctx.get_component_signal_content(access)?),
        _ => Err(ProgramError::InvalidDataType),
    }
}

/// Connects two composed signals
fn connect_signal_arrays(
    ac: &mut ArithmeticCircuit,
    a: &Vec<NestedValue<u32>>,
    b: &Vec<NestedValue<u32>>,
) -> Result<(), ProgramError> {
    // Verify that the arrays have the same length
    if a.len() != b.len() {
        return Err(ProgramError::InvalidDataType);
    }

    for (a, b) in a.iter().zip(b.iter()) {
        match (a, b) {
            (NestedValue::Value(a), NestedValue::Value(b)) => {
                ac.add_connection(*a, *b)?;
            }
            (NestedValue::Array(a), NestedValue::Array(b)) => {
                connect_signal_arrays(ac, a, b)?;
            }
            _ => return Err(ProgramError::InvalidDataType),
        }
    }

    Ok(())
}

/// Builds a DataAccess from an Access array
fn build_access(
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

/// Executes an operation on two u32 values, performing the specified arithmetic or logical computation.
fn execute_op(lhs: u32, rhs: u32, op: &ExpressionInfixOpcode) -> Result<u32, ProgramError> {
    let res = match op {
        ExpressionInfixOpcode::Mul => lhs * rhs,
        ExpressionInfixOpcode::Div => {
            if rhs == 0 {
                return Err(ProgramError::OperationError("Division by zero".to_string()));
            }

            lhs / rhs
        }
        ExpressionInfixOpcode::Add => lhs + rhs,
        ExpressionInfixOpcode::Sub => lhs - rhs,
        ExpressionInfixOpcode::Pow => lhs.pow(rhs),
        ExpressionInfixOpcode::IntDiv => {
            if rhs == 0 {
                return Err(ProgramError::OperationError(
                    "Integer division by zero".to_string(),
                ));
            }

            lhs / rhs
        }
        ExpressionInfixOpcode::Mod => {
            if rhs == 0 {
                return Err(ProgramError::OperationError("Modulo by zero".to_string()));
            }

            lhs % rhs
        }
        ExpressionInfixOpcode::ShiftL => lhs << rhs,
        ExpressionInfixOpcode::ShiftR => lhs >> rhs,
        ExpressionInfixOpcode::LesserEq => {
            if lhs <= rhs {
                1
            } else {
                0
            }
        }
        ExpressionInfixOpcode::GreaterEq => {
            if lhs >= rhs {
                1
            } else {
                0
            }
        }
        ExpressionInfixOpcode::Lesser => {
            if lhs < rhs {
                1
            } else {
                0
            }
        }
        ExpressionInfixOpcode::Greater => {
            if lhs > rhs {
                1
            } else {
                0
            }
        }
        ExpressionInfixOpcode::Eq => {
            if lhs == rhs {
                1
            } else {
                0
            }
        }
        ExpressionInfixOpcode::NotEq => {
            if lhs != rhs {
                1
            } else {
                0
            }
        }
        ExpressionInfixOpcode::BoolOr => {
            if lhs != 0 || rhs != 0 {
                1
            } else {
                0
            }
        }
        ExpressionInfixOpcode::BoolAnd => {
            if lhs != 0 && rhs != 0 {
                1
            } else {
                0
            }
        }
        ExpressionInfixOpcode::BitOr => lhs | rhs,
        ExpressionInfixOpcode::BitAnd => lhs & rhs,
        ExpressionInfixOpcode::BitXor => lhs ^ rhs,
    };

    Ok(res)
}
