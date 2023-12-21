//! # Traverse Module
//!
//! This module provides functionality for traversing statements, expressions, infix operations and declaration of components, signals and variables.
//!
//! It's main purpose is to traverse signals.

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::execute::{execute_expression, execute_statement};
use crate::runtime::{DataContent, Runtime};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{
    Access, Expression, ExpressionInfixOpcode, SignalType, Statement, VariableType,
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
) {
    for statement in statements {
        traverse_statement(ac, runtime, statement, program_archive);
    }
    // TODO: handle complete template
}

/// Analyzes a single statement, delegating to specialized functions based on the statement's nature.
pub fn traverse_statement(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    stmt: &Statement,
    program_archive: &ProgramArchive,
) {
    match stmt {
        Statement::InitializationBlock {
            initializations, ..
        } => {
            for statement in initializations {
                traverse_statement(ac, runtime, statement, program_archive);
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
                .map(|dimension| {
                    let (dim_u32_str, _) =
                        execute_expression(ac, runtime, name, dimension, program_archive);
                    dim_u32_str
                        .parse::<u32>()
                        .expect("Failed to parse dimension")
                })
                .collect();

            match xtype {
                VariableType::Component => {
                    traverse_component_declaration(ac, runtime, name, &dim_u32_vec)
                }
                VariableType::Var => traverse_variable_declaration(ac, runtime, name, &dim_u32_vec),
                VariableType::Signal(signal_type, _tag_list) => {
                    traverse_signal_declaration(ac, runtime, name, *signal_type, &dim_u32_vec)
                }
                _ => unimplemented!(),
            }
        }
        Statement::While { cond, stmt, .. } => loop {
            let var = String::from("while");
            let (res, rb) = execute_expression(ac, runtime, &var, cond, program_archive);
            if res.contains('0') {
                break;
            }
            debug!("While res = {} {}", res, rb);
            traverse_statement(ac, runtime, stmt, program_archive);
        },
        Statement::IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            let var = String::from("IFTHENELSE");
            let (res, _) = execute_expression(ac, runtime, &var, cond, program_archive);
            let else_case = else_case.as_ref().map(|e| e.as_ref());
            if res.contains('0') {
                if let Option::Some(else_stmt) = else_case {
                    traverse_statement(ac, runtime, else_stmt, program_archive);
                }
            } else {
                traverse_statement(ac, runtime, if_case, program_archive)
            }
        }
        Statement::Substitution {
            var, access, rhe, ..
        } => {
            let mut name_access = String::from(var);
            debug!("Sub Variable found {}", var.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Sub Array access found");
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        name_access.push('_');
                        name_access.push_str(dim_u32_str.as_str());
                        debug!("Sub Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(_) => {
                        todo!("Sub Component access not handled");
                    }
                }
            }
            let rhs = traverse_expression(ac, runtime, &name_access, rhe, program_archive);
            debug!("Sub Assigning {} to {}", rhs, &name_access);
            execute_statement(ac, runtime, stmt, program_archive);
        }
        Statement::Block { stmts, .. } => {
            traverse_sequence_of_statements(ac, runtime, stmts, program_archive, true);
        }
        _ => unimplemented!("Statement not implemented"),
    }
}

/// Examines an expression to determine its structure and dependencies before execution.
pub fn traverse_expression(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    var: &String,
    expression: &Expression,
    _program_archive: &ProgramArchive,
) -> String {
    match expression {
        Expression::Number(_, value) => {
            // Declaring a constant.
            let val = value.to_u32().unwrap();
            debug!("Number value {}", val);

            let res = runtime.get_current_context().unwrap().declare_const(val);
            if res.is_ok() {
                ac.add_const_var(val, val); // Setting as id the constant value
            }

            val.to_string()
        }
        Expression::InfixOp {
            lhe, infix_op, rhe, ..
        } => {
            let ctx = runtime.get_current_context().unwrap();
            //TODO: for generic handling we should generate a name for an intermediate expression, we could ideally use only the values returned
            let varlhs = ctx.declare_auto_var().unwrap();
            debug!("Auto var for lhs {}", varlhs);
            let varrhs = ctx.declare_auto_var().unwrap();
            debug!("Auto var for rhs {}", varrhs);
            let varlop = traverse_expression(ac, runtime, &varlhs, lhe, _program_archive);
            debug!("lhs {}", varlop);
            let varrop = traverse_expression(ac, runtime, &varrhs, rhe, _program_archive);
            debug!("rhs {}", varlop);
            let (res, ret) = traverse_infix_op(ac, runtime, var, &varlop, &varrop, infix_op);
            if ret {
                return res.to_string();
            }
            var.to_string()
        }
        Expression::PrefixOp { .. } => {
            debug!("Prefix found");
            var.to_string()
        }
        Expression::Variable { name, access, .. } => {
            let mut name_access = String::from(name);
            debug!("Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, _program_archive);
                        name_access.push('_');
                        name_access.push_str(dim_u32_str.as_str());
                        debug!("Changed var name to {}", name_access);
                    }
                    Access::ComponentAccess(_) => {
                        debug!("Component access found");
                        todo!()
                    }
                }
            }
            let ctx = runtime.get_current_context().unwrap();
            if ctx.get_data_item(&name_access).is_ok() {
                let data_item = ctx.get_data_item(&name_access).unwrap();
                if data_item.get_content().is_some() {
                    // We're assuming data item is not an array
                    if let DataContent::Scalar(val) = data_item.get_content().unwrap() {
                        // TODO: Check if this is a constant
                        let cloned_val = *val;
                        debug!("Return var value {} = {}", name_access, val);
                        ctx.declare_const(cloned_val).unwrap();
                        ac.add_const_var(cloned_val, cloned_val);
                        return cloned_val.to_string();
                    }
                }
            }
            name_access.to_string()
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
                let (_, _) = execute_expression(ac, runtime, arg_name, arg_value, _program_archive);
                // TODO: set res to arg_name
            }

            // HERE IS CODE FOR FUNCTIGON

            let fn_body = _program_archive.get_function_data(id).get_body_as_vec();
            traverse_sequence_of_statements(ac, runtime, fn_body, _program_archive, true);

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
            id.to_string()
        }
        Expression::ArrayInLine { .. } => {
            debug!("ArrayInLine found");
            var.to_string()
        }
        Expression::UniformArray { .. } => {
            debug!("UniformArray found");
            var.to_string()
        }
        _ => unimplemented!("Expression not implemented"),
    }
}

/// Prepares an infix operation (like addition, subtraction) for execution by analyzing its components.
pub fn traverse_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    output: &str,
    input_lhs: &str,
    input_rhs: &str,
    infixop: &ExpressionInfixOpcode,
) -> (u32, bool) {
    debug!("Traversing infix op");
    let ctx = runtime.get_current_context().unwrap();

    // Check availability of lhs and rhs values
    let lhsvar_res = ctx.get_data_item(input_lhs);
    let rhsvar_res = ctx.get_data_item(input_rhs);

    // TODO: HERE IS WHERE INFIX OP CREATE GATES BUT BECAUSE WE CAN GET VALUES FROM lhs and rhs IT WILL EXECUTE INSTEAD
    // Skip traversal if can execute
    // if lhsvar_res.is_ok() && rhsvar_res.is_ok() {
    //     return execute_infix_op(ac, runtime, output, input_lhs, input_rhs, infixop);
    // }
    // debug!("Can't get variables: lhs={}, rhs={}", input_lhs, input_rhs);
    // let _ = ctx.clear_data_item(output).unwrap();

    // Traverse the infix operation
    let lhsvar_id = lhsvar_res.unwrap().get_u32().unwrap();
    let rhsvar_id = rhsvar_res.unwrap().get_u32().unwrap();
    let output_id = ctx.get_data_item(output).unwrap().get_u32().unwrap();

    let gate_type = AGateType::from(infixop);
    debug!("{} = {} {} {}", output, input_lhs, gate_type, input_rhs);

    ac.add_gate(output, output_id, lhsvar_id, rhsvar_id, gate_type);

    (0, false)
}

/// Processes the declaration of a component.
pub fn traverse_component_declaration(
    _ac: &mut ArithmeticCircuit,
    _runtime: &mut Runtime,
    _comp_name: &str,
    _dim_u32_vec: &[u32],
) {
    todo!()
}

/// Processes a signal declaration, integrating it into the circuit's variable management system.
pub fn traverse_signal_declaration(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    signal_name: &str,
    _signal_type: SignalType,
    dim_u32_vec: &Vec<u32>,
) {
    traverse_variable_declaration(ac, runtime, signal_name, dim_u32_vec);
}

/// Handles the declaration of variables, allocating and initializing them within the circuit.
pub fn traverse_variable_declaration(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    var_name: &str,
    dim_u32_vec: &Vec<u32>,
) {
    // Assuming this is a signal
    let ctx = runtime.get_current_context().unwrap();
    if dim_u32_vec.is_empty() {
        let signal_id = ctx.declare_signal(var_name).unwrap();
        ac.add_var(signal_id, var_name.to_string().as_str());
    } else {
        let dim_u32 = *dim_u32_vec.last().unwrap();
        for i in 0..dim_u32 {
            let (name, id) = ctx.declare_signal_array(var_name, vec![i]).unwrap();
            ac.add_var(id, &name);
        }
    }
}
