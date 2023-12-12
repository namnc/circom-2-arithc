//! # Runtime Module
//!
//! This module manages the runtime environment for arithmetic circuit computation, handling variable and execution context tracking.

use log::debug;
use rand::Rng;
use std::collections::HashMap;
use thiserror::Error;

/// New context origin
pub enum ContextOrigin {
    Call,
    Branch,
    Loop,
    Block,
}

/// Data type
#[derive(Clone)]
pub enum DataType {
    Signal,
    Variable,
}

/// Runtime - manages the scope stack and variable tracking.
pub struct Runtime {
    current_scope: u32,
    scopes: Vec<Scope>,
}

impl Runtime {
    /// Constructs a new Runtime with an empty stack.
    pub fn new() -> Self {
        Self {
            current_scope: u32::default(),
            scopes: Vec::new(),
        }
    }

    /// Retrieves a specific scope by its ID.
    pub fn get_scope(&mut self, id: u32) -> Result<&mut Scope, RuntimeError> {
        let index = self.scopes.iter().position(|scope| scope.id == id);

        match index {
            Some(i) => Ok(&mut self.scopes[i]),
            None => Err(RuntimeError::ScopeRetrievalError),
        }
    }

    /// TODO: FIX, this should retreive the current scope with id, not the last one in the stack.
    /// Retrieves the current runtime context.
    pub fn get_current_scope(&mut self) -> Result<&mut Scope, RuntimeError> {
        self.scopes.last_mut().ok_or(RuntimeError::EmptyScopeStack)
    }

    /// Creates a new context for a function call or similar operation.
    pub fn add_scope(&mut self, origin: ContextOrigin) -> Result<(), RuntimeError> {
        // Retrieve the current context
        let current_scope = self.get_current_scope()?;

        // Create the new context and add it to the stack
        let mut new_scope = Scope::new(self)?;

        self.scopes.push(new_scope);
        self.current_scope = new_scope.id;

        Ok(())
    }

    /// Ends the current context and returns variables to the caller.
    // pub fn end_current_context(&mut self) -> Result<(), RuntimeError> {
    //     // Pop the current context off the stack
    //     if let Some(mut current_context) = self.ctx_stack.pop() {
    //         // Return variables to the caller
    //         let result = current_context.return_to_caller(self);

    //         // If there's an error, push the context back to restore the state
    //         if result.is_err() {
    //             self.ctx_stack.push(current_context);
    //         }

    //         result
    //     } else {
    //         Err(RuntimeError::EmptyContextStack)
    //     }
    // }

    // If first then else, so if 1 context -> if, if 2 contexts -> if else
    /// Merges the current branches and returns variables to the caller.
    pub fn merge_branches(&mut self) -> Result<(), RuntimeError> {
        todo!()
    }

    /// Retrieves the value of a variable from the current context.
    pub fn get_var(&mut self, var: &str) -> Result<u32, RuntimeError> {
        todo!()
    }

    /// Declares a variable in the current context and returns its identifier.
    pub fn declare_var(&mut self, name: &str) -> Result<u32, RuntimeError> {
        todo!()
    }
}

/// Runtime scope
/// Handles a specific scope value tracking.
#[derive(Clone)]
pub struct Scope { // Context
    id: u32,
    parent_id: u32,
    values: HashMap<String, DataItem<DataType>>, // Name -> Value
}

impl Scope {
    /// Constructs a new Scope.
    pub fn new(runtime: &mut Runtime) -> Result<Self, RuntimeError> {
        // Load the parent scope
        let mut rng = rand::thread_rng();
        let parent_scope = runtime.get_current_scope()?;

        Ok(Self {
            id: rng.gen(),
            parent_id: parent_scope.id,
            values: parent_scope.values.clone(),
        })
    }

    /// Declares a value in the scope
    pub fn declare_data_item(&mut self, name: &str) {
        todo!()
    }

    /// Assigns a value to a data item in the scope.
    /// If the data item is not declared, it will return an error.
    pub fn set_data_item(&mut self, name: &str, value: u32) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Clone)]
pub struct DataItem<T> {
    data_type: T,
    content: Option<DataContent>,
}

#[derive(Clone, Debug)]
pub enum DataContent {
    Scalar(u32),
    Array(Vec<DataContent>),
}

/// Runtime errors
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Error retrieving scope")]
    ScopeRetrievalError,
    #[error("Empty scope stack")]
    EmptyScopeStack,
    #[error("Variable is already declared")]
    VariableAlreadyDeclared,
    #[error("Variable is not declared")]
    VariableNotDeclared,
}

// /// Runtime Context
// /// Handles a specific runtime context, including variable tracking and execution context.
// #[derive(Clone)]
// pub struct RuntimeContext {
//     caller_id: u32,
//     context_id: u32,
//     execution: ExecutionContext,
//     var_ids: HashMap<String, u32>, // Variable Name -> Variable ID
// }

// impl RuntimeContext {

//     /// Returns variable changes to the caller context.
//     pub fn return_to_caller(&mut self, runtime: &mut CircomRuntime) -> Result<(), RuntimeError> {
//         // Load the caller context
//         let runtime_ctx = runtime.get_context(self.caller_id)?;

//         // Declare all variables in the caller context
//         for (name, &id) in &self.var_ids {
//             runtime_ctx.declare_var(name, id);
//         }

//         // Return all variables to the caller context
//         self.execution.return_to_caller(runtime_ctx)
//     }

//     /// Assigns a variable identifier and declares it in the execution context.
//     pub fn declare_var(&mut self, name: &str, id: u32) {
//         // Set the variable id
//         self.var_ids.insert(name.to_string(), id);
//         debug!("[RuntimeContext] {} is now with id {}", name, id);

//         // Declare the variable in the execution context
//         self.execution.declare_var(name);
//     }

//     /// Assigns a value to a variable in the execution context.
//     /// If the variable is not declared, it will return an error.
//     pub fn set_var(&mut self, name: &str, value: u32) -> Result<(), RuntimeError> {
//         if !self.var_ids.contains_key(name) {
//             return Err(RuntimeError::VariableNotDeclared);
//         }

//         self.execution.set_var(name, value)
//     }

//     /// Unsets a variable in the execution context.
//     /// If the variable is not declared, it will return an error.
//     pub fn unset_var(&mut self, name: &str) -> Result<(), RuntimeError> {
//         if !self.var_ids.contains_key(name) {
//             return Err(RuntimeError::VariableNotDeclared);
//         }

//         self.execution.unset_var(name)
//     }

//     /// Gets the value of a variable from the execution context.
//     /// If the variable is not declared, it will return an error.
//     pub fn get_value(&self, name: &str) -> Result<u32, RuntimeError> {
//         if !self.var_ids.contains_key(name) {
//             return Err(RuntimeError::VariableNotDeclared);
//         }

//         match self.execution.get_var(name)? {
//             Some(value) => Ok(value),
//             None => Ok(0),
//         }
//     }

//     /// Gets the id of a variable in the runtime context.
//     /// If the variable is not declared, it will return an error.
//     pub fn get_var_id(&self, var_name: &str) -> Result<u32, RuntimeError> {
//         self.var_ids
//             .get(var_name)
//             .copied()
//             .ok_or(RuntimeError::VariableNotDeclared)
//     }
// }

// // TODO: add signal and variable type
// // Split signal and variable

// /// Execution Context
// /// Handles variable operations and values for a specific runtime context.
// #[derive(Clone)]
// pub struct ExecutionContext {
//     vars: HashMap<String, Option<u32>>, // Variable Name -> Variable Value
// }

// impl ExecutionContext {
//     /// Constructs a new runtime execution context with specified caller and context IDs.
//     pub fn new(caller_id: u32, context_id: u32) -> Self {
//         Self {
//             vars: HashMap::new(),
//         }
//     }

//     /// Clones all variables from the specified context into this context.
//     pub fn load_context(&mut self, context: &RuntimeContext) -> &mut Self {
//         self.vars = context.execution.vars.clone();
//         self
//     }

//     /// Updates all variables from this context back to the caller's context.
//     pub fn return_to_caller(&mut self, context: &mut RuntimeContext) -> Result<(), RuntimeError> {
//         for (name, &val) in &self.vars {
//             match val {
//                 Some(value) => context.execution.set_var(name, value)?,
//                 None => context.execution.unset_var(name)?,
//             }
//         }
//         Ok(())
//     }

//     /// Declares a new variable in the context without setting its value (initialized as unset).
//     /// If the variable is already declared, it will be overwritten.
//     pub fn declare_var(&mut self, var_name: &str) {
//         self.vars.insert(var_name.to_owned(), None);
//         debug!("[ExecutionContext] '{}' is declared", var_name);
//     }

//     /// Retrieves the value of a variable if it is set, or None if it is unset.
//     /// If the variable is not declared, it will return an error.
//     pub fn get_var(&self, var_name: &str) -> Result<Option<u32>, RuntimeError> {
//         match self.vars.get(var_name) {
//             Some(&value) => Ok(value),
//             None => Err(RuntimeError::VariableNotDeclared),
//         }
//     }

//     /// Sets the value of a declared variable, marking it as set.
//     /// If the variable is not declared, it will return an error.
//     pub fn set_var(&mut self, var_name: &str, var_val: u32) -> Result<(), RuntimeError> {
//         match self.vars.get_mut(var_name) {
//             Some(value) => {
//                 *value = Some(var_val);
//                 debug!("[ExecutionContext] '{}' set to {}", var_name, var_val);
//                 Ok(())
//             }
//             None => Err(RuntimeError::VariableNotDeclared),
//         }
//     }

//     /// Unsets (clears) a specified variable.
//     /// If the variable is not declared, it will return an error.
//     pub fn unset_var(&mut self, var_name: &str) -> Result<(), RuntimeError> {
//         match self.vars.get_mut(var_name) {
//             Some(value) => {
//                 *value = None;
//                 debug!("[ExecutionContext] '{}' is unset", var_name);
//                 Ok(())
//             }
//             None => Err(RuntimeError::VariableNotDeclared),
//         }
//     }
// }
