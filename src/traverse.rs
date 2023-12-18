//! # Traverse Module
//!
//! This module provides functionality for traversing statements, expressions, infix operations and declaration of components, signals and variables.
//!
//! It's main purpose is to traverse signals.

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::execute::{execute_expression, execute_infix_op, execute_statement};
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
    stmts: &[Statement],
    program_archive: &ProgramArchive,
    is_complete_template: bool,
) {
    for stmt in stmts.iter() {
        traverse_statement(ac, runtime, stmt, program_archive);
    }
    if is_complete_template {
        //execute_delayed_declarations(program_archive, runtime, actual_node, flags)?;
    }
}

/// Analyzes a single statement, delegating to specialized functions based on the statement's nature.
pub fn traverse_statement(
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
                traverse_statement(ac, runtime, istmt, program_archive);
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
            if res.contains("0") {
                break;
            }
            debug!("While res = {} {}", res, rb);
            traverse_statement(ac, runtime, stmt, program_archive);
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
            debug!("Sub Variable found {}", var.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        debug!("Sub Array access found");
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        debug!("Sub Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        debug!("Sub Component access not handled");
                    }
                }
            }
            let rhs = traverse_expression(ac, runtime, &name_access, rhe, program_archive);
            debug!("Sub Assigning {} to {}", rhs, &name_access);
            execute_statement(ac, runtime, stmt, program_archive);
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

/// Examines an expression to determine its structure and dependencies before execution.
pub fn traverse_expression(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    var: &String,
    expr: &Expression,
    program_archive: &ProgramArchive,
) -> String {
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

            val.to_string()
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
            let varlop = traverse_expression(ac, runtime, &varlhs, lhe, program_archive);
            debug!("lhs {}", varlop);
            let varrop = traverse_expression(ac, runtime, &varrhs, rhe, program_archive);
            debug!("rhs {}", varlop);
            let (res, ret) = traverse_infix_op(ac, runtime, var, &varlop, &varrop, *infix_op);
            if ret {
                return res.to_string();
            }
            var.to_string()
        }
        PrefixOp {
            meta,
            prefix_op,
            rhe,
        } => {
            debug!("Prefix found ");
            var.to_string()
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
            let ctx = runtime.get_current_context().unwrap();
            if ctx.get_data_item(&name_access).is_ok() {
                let data_item = ctx.get_data_item(&name_access).unwrap();
                // We're assuming data item is not an array
                if let DataContent::Scalar(val) = data_item.get_content().unwrap() {
                    // TODO: Check if this is a constant
                    let cloned_val = val.clone();
                    debug!("Return var value {} = {}", name_access, cloned_val);
                    ctx.declare_const(cloned_val).unwrap();
                    ac.add_const_var(cloned_val, cloned_val);
                    return cloned_val.to_string();
                }
            }
            name_access.to_string()
        }
        Call { meta, id, args } => {
            debug!("Call found {}", id.to_string());
            // find the template and execute it
            id.to_string()
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
            var.to_string()
        }
        Tuple { meta, values } => todo!(),
        UniformArray {
            meta,
            value,
            dimension,
        } => {
            debug!("UniformArray found");
            var.to_string()
        }
    }
}

/// Prepares an infix operation (like addition, subtraction) for execution by analyzing its components.
pub fn traverse_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    output: &str,
    input_lhs: &str,
    input_rhs: &str,
    infixop: ExpressionInfixOpcode,
) -> (u32, bool) {
    debug!("Traversing infix op");
    let ctx = runtime.get_current_context().unwrap();

    // Check availability of lhs and rhs values
    let lhsvar_res = ctx.get_data_item(input_lhs);
    let rhsvar_res = ctx.get_data_item(input_rhs);

    // Skip traversal if can execute
    if lhsvar_res.is_ok() && rhsvar_res.is_ok() {
        return execute_infix_op(ac, runtime, output, input_lhs, input_rhs, infixop);
    }

    // Unreachable code

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
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    comp_name: &str,
    dim_u32_vec: &Vec<u32>,
) {
    // let var_id = runtime.assign_var_to_current_context(&var_name.to_string());
    // ac.add_var(var_id, comp_name.to_string().as_str());
    debug!("Found component {}", comp_name);
}

/// Processes a signal declaration, integrating it into the circuit's variable management system.
pub fn traverse_signal_declaration(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    signal_name: &str,
    signal_type: SignalType,
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
        // let mut all_accesses = Vec::new();
        // for u32s in dim_u32_vec.iter() {
        //     let mut accesses = Vec::new();
        //     for i in 0..*u32s {
        //         accesses.push(i);
        //     }
        //     all_accesses.push(accesses);
        // }
        // for accesses in all_accesses.iter() {

        // }
        let dim_u32 = *dim_u32_vec.last().unwrap();
        for i in 0..dim_u32 {
            let (name, id) = ctx
                .declare_signal_array(&var_name.to_string(), vec![i])
                .unwrap();
            ac.add_var(id, &name);
        }
    }
}
