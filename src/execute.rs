//! # Execute Module
//!
//! This module provides functionality to handle variables execution (not signals).

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::runtime::{DataContent, DataType, Runtime};
use crate::traverse::{
    traverse_component_declaration, traverse_sequence_of_statements, traverse_signal_declaration,
    traverse_variable_declaration,
};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{
    Access, Expression, ExpressionInfixOpcode, Statement, VariableType,
};
use circom_program_structure::program_archive::ProgramArchive;
use log::debug;

/// Executes a given statement, applying its logic or effects within the circuit's context.
pub fn execute_statement(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    statement: &Statement,
    program_archive: &ProgramArchive,
) {
    match statement {
        Statement::InitializationBlock {
            initializations, ..
        } => {
            for stmt in initializations {
                execute_statement(ac, runtime, stmt, program_archive);
            }
        }
        // TODO: THIS ACTUALLY WILL NOT HAPPEN IN EXECUTION
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
                        .expect("Failed to parse dimension as u32")
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
            debug!("res = {} {}", res, rb);
            execute_statement(ac, runtime, stmt, program_archive);
            if res.contains('0') {
                break;
            }
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
                    execute_statement(ac, runtime, else_stmt, program_archive);
                }
            } else {
                execute_statement(ac, runtime, if_case, program_archive)
            }
        }
        Statement::Substitution {
            var, access, rhe, ..
        } => {
            let mut name_access = String::from(var);
            debug!("Variable found {}", var.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let (dim_u32_str, _) =
                            execute_expression(ac, runtime, var, expr, program_archive);
                        name_access.push('_');
                        name_access.push_str(dim_u32_str.as_str());
                        debug!("Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(_) => {
                        todo!("Component access not handled");
                    }
                }
            }
            let (rhs, rhsb) = execute_expression(ac, runtime, &name_access, rhe, program_archive);
            debug!("Assigning {} ? {} to {}", rhs, rhsb, &name_access);
            if rhsb {
                debug!("Assigning {} to {}", rhs, &name_access);
                // TODO: revisit this
                // Check if the var already has this value assigned. If it doesn't, assign it.
                let ctx = runtime.get_current_context().unwrap();
                let res = ctx.get_data_item(&name_access);
                let expected: u32 = rhs.parse().unwrap();

                if res.is_ok() {
                    if expected == res.unwrap().get_u32().unwrap() {
                        debug!("Signal {} already has value {}", &name_access, expected);
                    } else {
                        ctx.clear_data_item(&name_access).unwrap();
                    }
                } else {
                    ctx.declare_data_item(&name_access, DataType::Variable)
                        .unwrap();
                    ctx.set_data_item(&name_access, DataContent::Scalar(expected))
                        .unwrap();
                }
            }
        }
        Statement::Return { value, .. } => {
            println!("Return expression found");
            let var = String::from("RETURN");
            let (res, _) = execute_expression(ac, runtime, &var, value, program_archive);
            println!("RETURN {}", res);
        }
        Statement::Block { stmts, .. } => {
            traverse_sequence_of_statements(ac, runtime, stmts, program_archive, true);
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
    var: &String,
    expr: &Expression,
    program_archive: &ProgramArchive,
) -> (String, bool) {
    match expr {
        Expression::Number(_, value) => {
            // Declaring a constant.
            let val = value.to_u32().unwrap();
            debug!("Number value {}", val);

            let res = runtime.get_current_context().unwrap().declare_const(val);
            if res.is_ok() {
                // Setting as id the constant value
                ac.add_const_var(val, val);
            }

            (val.to_string(), true)
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
            let (varlop, lhsb) = execute_expression(ac, runtime, &varlhs, lhe, program_archive);
            debug!("lhs {} {}", varlop, lhsb);
            let (varrop, rhsb) = execute_expression(ac, runtime, &varrhs, rhe, program_archive);
            debug!("rhs {} {}", varrop, rhsb);
            let (res, rb) = execute_infix_op(ac, runtime, var, &varlop, &varrop, infix_op);
            debug!("infix out res {}", res);
            (res.to_string(), rb)
        }
        Expression::PrefixOp { .. } => {
            debug!("Prefix found ");
            (var.to_string(), false)
        }
        Expression::Variable { name, access, .. } => {
            let mut name_access = String::from(name);
            debug!("Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let (dim_u32_str, _) =
                            execute_expression(ac, runtime, var, expr, program_archive);
                        name_access.push('_');
                        name_access.push_str(dim_u32_str.as_str());
                        debug!("Changed var name to {}", name_access);
                    }
                    Access::ComponentAccess(_) => {
                        todo!("Component access found");
                    }
                }
            }
            (name_access.to_string(), false)
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
                let (_, _) = execute_expression(ac, runtime, arg_name, arg_value, program_archive);
                // TODO: set res to arg_name
            }

            // HERE IS CODE FOR FUNCTIGON

            let fn_body = program_archive.get_function_data(id).get_body_as_vec();
            traverse_sequence_of_statements(ac, runtime, fn_body, program_archive, true);

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
            (id.to_string(), false)
        }
        Expression::ArrayInLine { .. } => {
            debug!("ArrayInLine found");
            (var.to_string(), false)
        }
        Expression::UniformArray { .. } => {
            debug!("UniformArray found");
            (var.to_string(), false)
        }
        _ => unimplemented!(),
    }
}

/// Executes an infix operation, performing the specified arithmetic or logical computation.
pub fn execute_infix_op(
    _ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    output: &str,
    input_lhs: &str,
    input_rhs: &str,
    infixop: &ExpressionInfixOpcode,
) -> (u32, bool) {
    debug!("Executing infix op");
    let ctx = runtime.get_current_context().unwrap();

    // Check availability of lhs and rhs values
    let lhsvar_res = ctx.get_data_item(input_lhs);
    let rhsvar_res = ctx.get_data_item(input_rhs);

    if lhsvar_res.is_err() || rhsvar_res.is_err() {
        debug!(
            "Error getting variables: lhs={}, rhs={}",
            input_lhs, input_rhs
        );
        ctx.clear_data_item(output).unwrap();
        return (0, false);
    }

    // Extract values
    let lhsvar_val = lhsvar_res.unwrap().get_u32().unwrap();
    let rhsvar_val = rhsvar_res.unwrap().get_u32().unwrap();

    // ctx.declare_signal(output).unwrap();
    let gate_type = AGateType::from(infixop);
    debug!("{} = {} {} {}", output, input_lhs, gate_type, input_rhs);

    let res = match gate_type {
        AGateType::AAdd => lhsvar_val + rhsvar_val,
        AGateType::ADiv => lhsvar_val / rhsvar_val,
        AGateType::AEq => {
            if lhsvar_val == rhsvar_val {
                1
            } else {
                0
            }
        }
        AGateType::AGEq => {
            if lhsvar_val >= rhsvar_val {
                1
            } else {
                0
            }
        }
        AGateType::AGt => {
            if lhsvar_val > rhsvar_val {
                1
            } else {
                0
            }
        }
        AGateType::ALEq => {
            if lhsvar_val <= rhsvar_val {
                1
            } else {
                0
            }
        }
        AGateType::ALt => {
            if lhsvar_val < rhsvar_val {
                1
            } else {
                0
            }
        }
        AGateType::AMul => lhsvar_val * rhsvar_val,
        AGateType::ANeq => {
            if lhsvar_val != rhsvar_val {
                1
            } else {
                0
            }
        }
        AGateType::ANone => todo!(),
        AGateType::ASub => lhsvar_val - rhsvar_val,
    };

    debug!("Infix res = {}", res);

    (res, true)
}
