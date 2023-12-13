//! # Execute Module
//!
//! This module provides functionality to handle variables execution (not signals).

use crate::circuit::{AGateType, ArithmeticCircuit};
use crate::runtime::{DataContent, Runtime};
use crate::traverse::{
    traverse_component_declaration, traverse_expression, traverse_sequence_of_statements,
    traverse_signal_declaration, traverse_variable_declaration,
};
use circom_circom_algebra::num_traits::ToPrimitive;
use circom_program_structure::ast::{
    Access, Expression, ExpressionInfixOpcode, Statement, VariableType,
};
use circom_program_structure::program_archive::ProgramArchive;

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
            println!("Declaration of {}", name);
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
            println!("[Execute] res = {} {}", res, rb);
            execute_statement(ac, runtime, stmt, program_archive);
            if res.contains("0") {
                break;
            }
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
            println!("[Execute] Variable found {}", var.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        println!("[Execute] Array access found");
                        // let mut dim_u32_vec = Vec::new();
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        // dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        println!("[Execute] Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        println!("Component access not handled");
                    }
                }
            }
            let (rhs, rhsb) = execute_expression(ac, runtime, &name_access, rhe, program_archive);
            println!("[Execute] Assigning {} ? {} to {}", rhs, rhsb, &name_access);
            if rhsb {
                println!("[Execute] Assigning {} to {}", rhs, &name_access);
                runtime
                    .get_current_context()
                    .unwrap()
                    .set_data_item(
                        &name_access,
                        DataContent::Scalar(rhs.parse::<u32>().unwrap()),
                    )
                    .unwrap();
            }
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
            //TODO: here we handle constant expression, we can do something like declare_if_not_exists for constant variables
            let var_id = runtime.declare_var(&value.to_string()).unwrap();
            runtime
                .set_var(&value.to_string(), value.to_u32().unwrap())
                .unwrap();
            ac.add_const_var(var_id, value.to_u32().unwrap());
            println!("[Execute] Number value {}", value);
            (value.to_string(), true)
        }
        InfixOp {
            meta,
            lhe,
            infix_op,
            rhe,
            ..
        } => {
            //TODO: for generic handling we should generate a name for an intermediate expression, we could ideally use only the values returned
            let varlhs = runtime.auto_generate_var().unwrap();
            println!("[Execute] Auto var for lhs {}", varlhs);
            let varrhs = runtime.auto_generate_var().unwrap();
            println!("[Execute] Auto var for rhs {}", varrhs);
            let (varlop, lhsb) = execute_expression(ac, runtime, &varlhs, lhe, program_archive);
            println!("[Execute] lhs {} {}", varlop, lhsb);
            let (varrop, rhsb) = execute_expression(ac, runtime, &varrhs, rhe, program_archive);
            println!("[Execute] rhs {} {}", varrop, rhsb);
            let (res, rb) = execute_infix_op(ac, runtime, var, &varlop, &varrop, *infix_op);
            println!("[Execute] infix out res {}", res);
            (res.to_string(), rb)
        }
        PrefixOp {
            meta,
            prefix_op,
            rhe,
        } => {
            println!("Prefix found ");
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
            println!("[Execute] Variable found {}", name.to_string());
            for a in access.iter() {
                match a {
                    Access::ArrayAccess(expr) => {
                        println!("[Execute] Array access found");
                        // let mut dim_u32_vec = Vec::new();
                        let dim_u32_str =
                            traverse_expression(ac, runtime, var, expr, program_archive);
                        // dim_u32_vec.push(dim_u32_str.parse::<u32>().unwrap());
                        name_access.push_str("_");
                        name_access.push_str(dim_u32_str.as_str());
                        println!("[Execute] Change var name to {}", name_access);
                    }
                    Access::ComponentAccess(name) => {
                        println!("Component access found");
                    }
                }
            }
            if runtime.get_var(&name_access).is_ok() {
                let var_val = runtime.get_var(&name_access).unwrap().to_string();
                println!("[Execute] Return var value {} = {}", name_access, var_val);
                runtime.declare_var(&var_val).unwrap();
                runtime
                    .set_var(&var_val, var_val.parse::<u32>().unwrap())
                    .unwrap();
                return (var_val, true);
            }
            (name_access.to_string(), false)
        }
        Call { meta, id, args } => {
            println!("Call found {}", id.to_string());
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
            println!("ArrayInLine found");
            (var.to_string(), false)
        }
        Tuple { meta, values } => todo!(),
        UniformArray {
            meta,
            value,
            dimension,
        } => {
            println!("UniformArray found");
            (var.to_string(), false)
        }
    }
}

//WIP HERE
// TODO: named_access should support multi-dimension, right now 1

/// Executes an infix operation, performing the specified arithmetic or logical computation.
pub fn execute_infix_op(
    ac: &mut ArithmeticCircuit,
    runtime: &mut Runtime,
    output: &String,
    input_lhs: &String,
    input_rhs: &String,
    infixop: ExpressionInfixOpcode,
) -> (u32, bool) {
    // let current = runtime.get_current_runtime_context();
    let mut can_execute_infix = true;
    if runtime.get_var(input_lhs).is_err() {
        println!("[Execute] cannot get lhs var val {}", input_lhs);
        can_execute_infix = false;
    }
    if runtime.get_var(input_rhs).is_err() {
        println!("[Execute] cannot get rhs var val {}", input_rhs);
        can_execute_infix = false;
    }
    println!("[Execute] can execute infix {}", can_execute_infix);

    if !can_execute_infix {
        runtime.unset_var(output).unwrap();
        println!("[Execute] Now mark {} as no value", output);
        return (0, false);
    }

    let lhsvar_val = runtime.get_var(input_lhs).unwrap();
    println!("[Execute] infix lhs = {}", lhsvar_val);
    let rhsvar_val = runtime.get_var(input_rhs).unwrap();
    println!("[Execute] infix lhs = {}", rhsvar_val);
    let var_id = runtime.declare_var(output).unwrap();

    // let var = ac.add_var(var_id, &output);

    // let lvar = ac.get_var(lhsvar_id);
    // let rvar = ac.get_var(rhsvar_id);

    let mut res = 0;

    use ExpressionInfixOpcode::*;
    let mut gate_type = AGateType::AAdd;
    match infixop {
        Mul => {
            println!(
                "[Execute] Mul op {} = {} * {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::AMul;
            res = lhsvar_val * rhsvar_val;
        }
        Div => {
            println!(
                "[Execute] Div op {} = {} / {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::ADiv;
            res = lhsvar_val / rhsvar_val;
        }
        Add => {
            println!(
                "[Execute] Add op {} = {} + {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::AAdd;
            res = lhsvar_val + rhsvar_val;
        }
        Sub => {
            println!(
                "[Execute] Sub op {} = {} - {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::ASub;
            res = lhsvar_val - rhsvar_val;
        }
        // Pow => {},
        // IntDiv => {},
        // Mod => {},
        // ShiftL => {},
        // ShiftR => {},
        LesserEq => {
            println!(
                "[Execute] LesserEq op {} = {} <= {}",
                output, lhsvar_val, rhsvar_val
            );
            res = if lhsvar_val <= rhsvar_val { 1 } else { 0 };
        }
        GreaterEq => {
            println!(
                "[Execute] GreaterEq op {} = {} >= {}",
                output, lhsvar_val, rhsvar_val
            );
            res = if lhsvar_val >= rhsvar_val { 1 } else { 0 };
        }
        Lesser => {
            println!(
                "[Execute] Lesser op {} = {} < {}",
                output, lhsvar_val, rhsvar_val
            );
            res = if lhsvar_val < rhsvar_val { 1 } else { 0 };
        }
        Greater => {
            println!(
                "[Execute] Greater op {} = {} > {}",
                output, lhsvar_val, rhsvar_val
            );
            res = if lhsvar_val > rhsvar_val { 1 } else { 0 };
        }
        Eq => {
            println!(
                "[Execute] Eq op {} = {} == {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::AEq;
            res = if lhsvar_val == rhsvar_val { 1 } else { 0 };
        }
        NotEq => {
            println!(
                "[Execute] Neq op {} = {} != {}",
                output, lhsvar_val, rhsvar_val
            );
            gate_type = AGateType::ANeq;
            res = if lhsvar_val != rhsvar_val { 1 } else { 0 };
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
    println!("[Execute] infix res = {}", res);
    (res, true)
    // ac.add_gate(&output, var_id, lhsvar_id, rhsvar_id, gate_type);
}
