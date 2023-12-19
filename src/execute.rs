//! # Execute Module
//!
//! This module provides functionality to handle variables execution (not signals).

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::runtime::{DataContent, DataType, Runtime};
use crate::traverse::{
    traverse_component_declaration, traverse_expression, traverse_sequence_of_statements,
    traverse_signal_declaration, traverse_variable_declaration,
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
    stmt: &Statement,
    program_archive: &ProgramArchive,
) {
    use Statement::*;
    let id = stmt.get_meta().elem_id;

    // Analysis::reached(&mut runtime.analysis, id);

    // let mut can_be_simplified = true;

    match stmt {
        InitializationBlock {
            initializations, ..
        } => {
            for istmt in initializations.iter() {
                execute_statement(ac, runtime, istmt, program_archive);
            }
        }
        Declaration {
            meta,
            xtype,
            name,
            dimensions,
            ..
        } => {
            debug!("Declaration of {}", name);
            match xtype {
                // VariableType::AnonymousComponent => {
                //     execute_anonymous_component_declaration(
                //         name,
                //         meta.clone(),
                //         &dimensions,
                //         &mut runtime.environment,
                //         &mut runtime.anonymous_components,
                //     );
                // }
                _ => {
                    // Process index in case of array
                    let mut dim_u32_vec = Vec::new();
                    for dimension in dimensions.iter() {
                        let dim_u32_str =
                            traverse_expression(ac, runtime, name, dimension, program_archive);
                        dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                    }
                    // treat_result_with_memory_error_void(
                    //     valid_array_declaration(&arithmetic_values),
                    //     meta,
                    //     &mut runtime.runtime_errors,
                    //     &runtime.call_trace,
                    // )?;
                    // let usable_dimensions =
                    //     if let Option::Some(dimensions) = cast_indexing(&arithmetic_values) {
                    //         dimensions
                    //     } else {
                    //         let err = Result::Err(ExecutionError::ArraySizeTooBig);
                    //         treat_result_with_execution_error(
                    //             err,
                    //             meta,
                    //             &mut runtime.runtime_errors,
                    //             &runtime.call_trace,
                    //         )?
                    //     };
                    match xtype {
                        VariableType::Component => traverse_component_declaration(
                            ac,
                            runtime,
                            name,
                            &dim_u32_vec, // &usable_dimensions
                                          // &mut runtime.environment,
                                          // actual_node
                        ),
                        VariableType::Var => traverse_variable_declaration(
                            ac,
                            runtime,
                            name,
                            &dim_u32_vec, // &usable_dimensions
                        ),
                        VariableType::Signal(signal_type, tag_list) => traverse_signal_declaration(
                            ac,
                            runtime,
                            name,
                            *signal_type,
                            &dim_u32_vec, // &usable_dimensions
                        ),
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
            // Option::None
        }
        IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            // let var = String::from("IFTHENELSE");
            // ac.add_var(&var, SignalType::Intermediate);
            // let lhs = traverse_expression(ac, &var, cond, program_archive);
            // traverse_statement(ac, &if_case, program_archive);
            // let else_case = else_case.as_ref().map(|e| e.as_ref());
            // traverse_statement(ac, else_case.unwrap(), program_archive);
            //     let else_case = else_case.as_ref().map(|e| e.as_ref());
            //     let (possible_return, can_simplify, _) = execute_conditional_statement(
            //         cond,
            //         if_case,
            //         else_case,
            //         program_archive,
            //         runtime,
            //         actual_node,
            //         flags
            //     )?;
            //     can_be_simplified = can_simplify;
            //     possible_return
            // }
            // While { cond, stmt, .. } => loop {
            //     let (returned, can_simplify, condition_result) = execute_conditional_statement(
            //         cond,
            //         stmt,
            //         Option::None,
            //         program_archive,
            //         runtime,
            //         actual_node,
            //         flags
            //     )?;
            //     can_be_simplified &= can_simplify;
            //     if returned.is_some() {
            //         break returned;
            //     } else if condition_result.is_none() {
            //         let (returned, _, _) = execute_conditional_statement(
            //             cond,
            //             stmt,
            //             None,
            //             program_archive,
            //             runtime,
            //             actual_node,
            //             flags
            //         )?;
            //         break returned;
            //     } else if !condition_result.unwrap() {
            //         break returned;
            //     }
        }
        While { cond, stmt, .. } => loop {
            let var = String::from("while");
            let (res, rb) = execute_expression(ac, runtime, &var, cond, program_archive);
            debug!("res = {} {}", res, rb);
            execute_statement(ac, runtime, stmt, program_archive);
            if res.contains("0") {
                break;
            }
            // traverse_expression(ac, runtime, var, cond, program_archive);
            // let var = String::from("while");
            // ac.add_var(&var, SignalType::Intermediate);
            // let lhs = traverse_expression(ac, runtime, &var, cond, program_archive);
            // debug!("While cond {}", lhs);
            // traverse_statement(ac, stmt, program_archive);
        },
        ConstraintEquality { meta, lhe, rhe, .. } => {
            // debug_assert!(actual_node.is_some());
            // let f_left = execute_expression(lhe, program_archive, runtime, flags)?;
            // let f_right = execute_expression(rhe, program_archive, runtime, flags)?;
            // let arith_left = safe_unwrap_to_arithmetic_slice(f_left, line!());
            // let arith_right = safe_unwrap_to_arithmetic_slice(f_right, line!());

            // let correct_dims_result = AExpressionSlice::check_correct_dims(&arith_left, &Vec::new(), &arith_right, true);
            // treat_result_with_memory_error_void(
            //     correct_dims_result,
            //     meta,
            //     &mut runtime.runtime_errors,
            //     &runtime.call_trace,
            // )?;
            // for i in 0..AExpressionSlice::get_number_of_cells(&arith_left){
            //     let value_left = treat_result_with_memory_error(
            //         AExpressionSlice::access_value_by_index(&arith_left, i),
            //         meta,
            //         &mut runtime.runtime_errors,
            //         &runtime.call_trace,
            //     )?;
            //     let value_right = treat_result_with_memory_error(
            //         AExpressionSlice::access_value_by_index(&arith_right, i),
            //         meta,
            //         &mut runtime.runtime_errors,
            //         &runtime.call_trace,
            //     )?;
            //     let possible_non_quadratic =
            //         AExpr::sub(
            //             &value_left,
            //             &value_right,
            //             &runtime.constants.get_p()
            //         );
            //     if possible_non_quadratic.is_nonquadratic() {
            //         treat_result_with_execution_error(
            //             Result::Err(ExecutionError::NonQuadraticConstraint),
            //             meta,
            //             &mut runtime.runtime_errors,
            //             &runtime.call_trace,
            //         )?;
            //     }
            //     let quadratic_expression = possible_non_quadratic;
            //     let constraint_expression = AExpr::transform_expression_to_constraint_form(
            //         quadratic_expression,
            //         runtime.constants.get_p(),
            //     )
            //     .unwrap();
            //     if let Option::Some(node) = actual_node {
            //         node.add_constraint(constraint_expression);
            //     }
            // }
            // Option::None
        }
        Return { value, .. } => {}
        Assert { arg, meta, .. } => {}
        Substitution {
            meta,
            var,
            access,
            op,
            rhe,
            ..
        } => {
            let mut name_access = String::from(var);
            debug!("Variable found {}", var.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        // let mut dim_u32_vec = Vec::new();
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        // dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        debug!("Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        debug!("Component access not handled");
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
        Block { stmts, .. } => {
            traverse_sequence_of_statements(ac, runtime, stmts, program_archive, true);
        }
        LogCall { args, .. } => {}
        UnderscoreSubstitution { meta, rhe, op } => {
            debug!("UnderscoreSubstitution found");
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
    use Expression::*;
    // let mut can_be_simplified = true;
    match expr {
        Number(_, value) => {
            // Declaring a constant.
            let val = value.to_u32().unwrap();
            debug!("Number value {}", val);

            let res = runtime.get_current_context().unwrap().declare_const(val);
            if res.is_ok() {
                res.unwrap();
                ac.add_const_var(val, val); // Setting as id the constant value
            }

            (val.to_string(), true)
        }
        InfixOp {
            meta,
            lhe,
            infix_op,
            rhe,
            ..
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
        PrefixOp {
            meta,
            prefix_op,
            rhe,
        } => {
            debug!("Prefix found ");
            (var.to_string(), false)
        }
        InlineSwitchOp {
            meta,
            cond,
            if_true,
            if_false,
        } => todo!(),
        ParallelOp { meta, rhe } => todo!(),
        Variable { meta, name, access } => {
            let mut name_access = String::from(name);
            debug!("Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Array access found");
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        debug!("Changed var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        debug!("Component access found");
                    }
                }
            }
            (name_access.to_string(), false)
        }
        Call { meta, id, args } => {
            debug!("Call found {}", id.to_string());
            // find the template and execute it
            (id.to_string(), false)
        }
        AnonymousComp {
            meta,
            id,
            is_parallel,
            params,
            signals,
            names,
        } => todo!(),
        ArrayInLine { meta, values } => {
            debug!("ArrayInLine found");
            (var.to_string(), false)
        }
        Tuple { meta, values } => todo!(),
        UniformArray {
            meta,
            value,
            dimension,
        } => {
            debug!("UniformArray found");
            (var.to_string(), false)
        }
    }
}

/// Executes an infix operation, performing the specified arithmetic or logical computation.
pub fn execute_infix_op(
    ac: &mut ArithmeticCircuit,
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
