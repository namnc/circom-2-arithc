//! # Traverse Module
//!
//! This module provides functionality for traversing statements, expressions, infix operations and declaration of components, signals and variables.
//!
//! It's main purpose is to traverse signals.

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::execute::{execute_expression, execute_infix_op, execute_statement};
use crate::runtime::{DataContent, DataType, Runtime};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{
    Access, Expression, ExpressionInfixOpcode, SignalType, Statement, VariableType,
};
use circom_program_structure::program_archive::ProgramArchive;

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
            println!("[Traverse] Declaration of {}", name);
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
            println!("[Traverse] While res = {} {}", res, rb);
            traverse_statement(ac, runtime, stmt, program_archive);
            // traverse_expression(ac, runtime, var, cond, program_archive);
            // let var = String::from("while");
            // ac.add_var(&var, SignalType::Intermediate);
            // let lhs = traverse_expression(ac, runtime, &var, cond, program_archive);
            // println!("While cond {}", lhs);
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
            println!("[Traverse] Sub Variable found {}", var.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        println!("[Traverse] Sub Array access found");
                        // let mut dim_u32_vec = Vec::new();
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        // dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        println!("[Traverse] Sub Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        println!("[Traverse] Sub Component access not handled");
                    }
                }
            }
            let rhs = traverse_expression(ac, runtime, &name_access, rhe, program_archive);
            println!("[Traverse] Sub Assigning {} to {}", rhs, &name_access);
            execute_statement(ac, runtime, stmt, program_archive);
        }
        Block { stmts, .. } => {
            traverse_sequence_of_statements(ac, runtime, stmts, program_archive, true);
        }
        LogCall { args, .. } => {}
        UnderscoreSubstitution { meta, rhe, op } => {
            println!("UnderscoreSubstitution found");
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
            let var_id = runtime
                .get_current_context()
                .unwrap()
                .declare_data_item(&value.to_string(), DataType::Variable)
                .unwrap();
            runtime
                .get_current_context()
                .unwrap()
                .set_data_item(
                    &value.to_string(),
                    DataContent::Scalar(value.to_u32().unwrap()),
                )
                .unwrap();
            // TODO: What should I do here? Is this a variable or a signal?
            // ac.add_const_var(var_id, value.to_u32().unwrap());
            println!("[Traverse] Number value {}", value);
            value.to_string()
        }
        InfixOp {
            meta,
            lhe,
            infix_op,
            rhe,
            ..
        } => {
            let varlhs = runtime.auto_generate_var().unwrap();
            println!("[Traverse] Auto var for lhs {}", varlhs);
            let varrhs = runtime.auto_generate_var().unwrap();
            println!("[Traverse] Auto var for rhs {}", varrhs);
            let varlop = traverse_expression(ac, runtime, &varlhs, lhe, program_archive);
            println!("[Traverse] lhs {}", varlop);
            let varrop = traverse_expression(ac, runtime, &varrhs, rhe, program_archive);
            println!("[Traverse] rhs {}", varlop);
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
            println!("Prefix found ");
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
            println!("[Traverse] Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        println!("[Traverse] Array access found");
                        // let mut dim_u32_vec = Vec::new();
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        // dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        println!("[Traverse] Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        println!("[Traverse] Component access found");
                    }
                }
            }
            if runtime.get_var(&name_access).is_ok() {
                let var_val = runtime.get_var(&name_access).unwrap().to_string();
                println!("[Traverse] Return var value {} = {}", name_access, var_val);
                let var_id = runtime.declare_var(&var_val).unwrap();
                let var_val_n = runtime
                    .set_var(&var_val, var_val.parse::<u32>().unwrap())
                    .unwrap();
                ac.add_const_var(var_id, var_val_n);
                return var_val.to_string();
            }
            name_access.to_string()
        }
        Call { meta, id, args } => {
            println!("Call found {}", id.to_string());
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
            println!("ArrayInLine found");
            var.to_string()
        }
        Tuple { meta, values } => todo!(),
        UniformArray {
            meta,
            value,
            dimension,
        } => {
            println!("UniformArray found");
            var.to_string()
        }
    }
}

/// Prepares an infix operation (like addition, subtraction) for execution by analyzing its components.
pub fn traverse_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    output: &String,
    input_lhs: &String,
    input_rhs: &String,
    infixop: ExpressionInfixOpcode,
) -> (u32, bool) {
    // let current = runtime.get_current_runtime_context();

    // For now skip traversal if can execute

    let mut can_execute_infix = true;
    if runtime.get_var(input_lhs).is_err() {
        println!("[Traverse] cannot get lhs var val {}", input_lhs);
        can_execute_infix = false;
    }
    if runtime.get_var(input_rhs).is_err() {
        println!("[Traverse] cannot get rhs var val {}", input_rhs);
        can_execute_infix = false;
    }
    println!("[Traverse] can execute infix {}", can_execute_infix);

    if can_execute_infix {
        return execute_infix_op(ac, runtime, output, input_lhs, input_rhs, infixop);
    } else {
        runtime.unset_var(output).unwrap();
        println!("[Traverse] Now mark {} as no value", output);
    }

    let lhsvar_id = runtime.get_var(input_lhs).unwrap();
    let rhsvar_id = runtime.get_var(input_rhs).unwrap();
    let var_id = runtime.get_var(output).unwrap();

    // let var = ac.add_var(var_id, &output);

    // let lvar = ac.get_var(lhsvar_id);
    // let rvar = ac.get_var(rhsvar_id);

    use ExpressionInfixOpcode::*;
    let mut gate_type = AGateType::AAdd;
    match infixop {
        Mul => {
            println!(
                "[Traverse] Mul op {} = {} * {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AMul;
        }
        Div => {
            println!(
                "[Traverse] Div op {} = {} / {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ADiv;
        }
        Add => {
            println!(
                "[Traverse] Add op {} = {} + {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AAdd;
        }
        Sub => {
            println!(
                "[Traverse] Sub op {} = {} - {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ASub;
        }
        // Pow => {},
        // IntDiv => {},
        // Mod => {},
        // ShiftL => {},
        // ShiftR => {},
        LesserEq => {
            println!(
                "[Traverse] LEq op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ALEq;
        }
        GreaterEq => {
            println!(
                "[Traverse] GEq op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AGEq;
        }
        Lesser => {
            println!(
                "[Traverse] Ls op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ALt;
        }
        Greater => {
            println!(
                "[Traverse] Gt op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AGt;
        }
        Eq => {
            println!(
                "[Traverse] Eq op {} = {} == {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::AEq;
        }
        NotEq => {
            println!(
                "[Traverse] Neq op {} = {} != {}",
                output, input_lhs, input_rhs
            );
            gate_type = AGateType::ANeq;
        }
        // BoolOr => {},
        // BoolAnd => {},
        // BitOr => {},
        // BitAnd => {},
        // BitXor => {},
        _ => {
            unreachable!()
        }
    };

    ac.add_gate(&output, var_id, lhsvar_id, rhsvar_id, gate_type);

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
    println!("Found component {}", comp_name);
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
    if dim_u32_vec.is_empty() {
        let var_id = runtime.declare_var(var_name).unwrap();
        ac.add_var(var_id, var_name.to_string().as_str());
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
            let mut u32vec = Vec::new();
            u32vec.push(i);
            let (var, var_id) = runtime
                .assign_array_var_to_current_context(&var_name.to_string(), u32vec)
                .unwrap();
            ac.add_var(var_id, var.as_str());
        }
    }
}
