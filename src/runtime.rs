//! # Runtime Module
//!
//! This module manages the runtime environment for arithmetic circuit computation, handling variable and execution context tracking.

use log::debug;
use std::collections::HashMap;
use thiserror::Error;

/// Circom Runtime
/// Handles the runtime context stack and variable tracking.
pub struct CircomRuntime {
    call_stack: Vec<RuntimeContext>,
    last_context_id: u32,
    last_var_id: u32,
}

impl CircomRuntime {
    /// Constructs a new CircomRuntime with an empty call stack and no variables.
    pub fn new() -> Self {
        Self {
            last_var_id: 0,
            last_context_id: 0,
            call_stack: vec![RuntimeContext::new(0, 0)],
        }
    }

    /// Retrieves a specific context by its ID.
    pub fn get_context(&mut self, id: u32) -> Result<&mut RuntimeContext, RuntimeError> {
        let index = self.call_stack.iter().position(|c| c.context_id == id);

        match index {
            Some(idx) => Ok(&mut self.call_stack[idx]),
            None => Err(RuntimeError::ContextRetrievalError),
        }
    }

    /// Retrieves the current runtime context.
    pub fn get_current_context(&self) -> Result<RuntimeContext, RuntimeError> {
        match self.call_stack.pop().as_mut() {
            Some(context) => Ok(context.clone()),
            None => Err(RuntimeError::EmptyContextStack),
        }
    }

    // Should add context from calling and branching
    // Notice that while loop and block also has local variable so to consider as calling

    /// Creates a new context for a function call or similar operation.
    pub fn new_context_from_calling(&mut self) {
        todo!()
    }

    // pub fn new_context_from_calling (&mut self) {
    //     self.last_context_id += 1;
    //     let mut rc = RuntimeContext::new(self.get_current_runtime_context_id(), self.last_context_id);
    //     rc.init(self);
    //     self.call_stack.push(rc);

    // }

    // pub fn new_context_from_branching (&mut self) {
    //     self.last_context_id += 1;
    //     let mut rc = RuntimeContext::new(self.get_current_runtime_context_caller_id(), self.last_context_id);
    //     rc.init(self);
    //     self.call_stack.push(rc);

    // }

    // pub fn end_current_context_return_vars (&self) {
    //     let rc = self.get_current_runtime_context();
    //     rc.return_to_caller(self);
    //     self.call_stack.pop();
    // }

    // If first then else, so if 1 context -> if, if 2 contexts -> if else
    // TODO: not handled for now
    // pub fn merge_current_branches_return_vars(&mut self) {
    //     let rc = self.get_current_runtime_context();
    //     rc.return_to_caller(self);
    //     self.call_stack.pop();
    // }

    // TODO
    // pub fn get_current_runtime_context_caller (&self) -> &RuntimeContext {
    //     let caller_id = self.get_current_runtime_context_caller_id();
    //     self.get_runtime_context_by_context_id(caller_id)

    // }

    // pub fn get_var_from_current_context(&self, var: &String) -> u32 {
    //     let current = self.get_current_runtime_context();
    //     current.get_var(var)
    // }
    // pub fn assign_var_to_current_context(&mut self, var: &String) -> u32 {
    //     self.last_var_id += 1;
    //     let var_id = self.last_var_id;
    //     let current = self.get_current_runtime_context_mut();
    //     current.assign_var(var, var_id)
    // }
    // pub fn can_get_var_val_from_current_context(&self, var: &String) -> bool {
    //     let current = self.get_current_runtime_context();
    //     current.can_get_var_val(var)
    // }
    // pub fn get_var_val_from_current_context(&self, var: &String) -> u32 {
    //     let current = self.get_current_runtime_context();
    //     current.get_var_val(var)
    // }
    // pub fn assign_var_val_to_current_context(&mut self, var: &String, var_val: u32) -> u32 {
    //     let current = self.get_current_runtime_context_mut();
    //     current.assign_var_val(var, var_val)
    // }

    // pub fn deassign_var_val_to_current_context(&mut self, var: &String) -> u32 {
    //     let current = self.get_current_runtime_context_mut();
    //     current.deassign_var_val(var)
    // }

    // pub fn assign_auto_var_to_current_context(&mut self) -> String {
    //     self.last_var_id += 1;
    //     let var_id = self.last_var_id;
    //     let current = self.get_current_runtime_context_mut();
    //     let var = format!("auto_var_{}", var_id);
    //     current.assign_var(&var, var_id);
    //     println!("[CircomRuntime] Auto var {}", var);
    //     var
    // }

    // TODO: array auto var should support multi-dimension, right now 1

    // pub fn assign_array_var_to_current_context(
    //     &mut self,
    //     var: &String,
    //     indice: Vec<u32>,
    // ) -> (String, u32) {
    //     self.last_var_id += 1;
    //     let var_id = self.last_var_id;
    //     let current = self.get_current_runtime_context_mut();
    //     let mut access_index = String::new();
    //     for i in 0..indice.len() {
    //         access_index.push_str(&format!("_{}", indice[i]));
    //     }
    //     let var = format!("{}{}", var, access_index);
    //     current.assign_var(&var, var_id);
    //     println!("[CircomRuntime] Array var {}", var);
    //     (var, var_id)
    // }
}

/// Runtime Context
/// Handles a specific runtime context, including variable tracking and execution context.
#[derive(Clone)]
pub struct RuntimeContext {
    caller_id: u32,
    context_id: u32,
    execution: ExecutionContext,
    var_ids: HashMap<String, u32>, // Variable Name -> Variable ID
}

impl RuntimeContext {
    /// Constructs a new RuntimeContext with specified caller and context IDs.
    pub fn new(caller_id: u32, context_id: u32) -> Self {
        Self {
            caller_id,
            context_id,
            var_ids: HashMap::new(),
            execution: ExecutionContext::new(caller_id, context_id),
        }
    }

    /// Initializes the runtime context with the caller context.
    pub fn init(&mut self, runtime: &CircomRuntime) -> Result<&mut Self, RuntimeError> {
        // Load the caller context
        let mut runtime_ctx = runtime.get_context(self.caller_id)?;

        // Copy the caller context's variable ids
        self.var_ids = runtime_ctx.var_ids.clone();

        // Copy the caller context's variables
        self.execution.load_context(runtime_ctx);

        Ok(self)
    }

    /// Returns variable changes to the caller context.
    pub fn return_to_caller(&mut self, runtime: &CircomRuntime) -> Result<(), RuntimeError> {
        // Load the caller context
        let mut runtime_ctx = runtime.get_context(self.caller_id)?;

        // Declare all variables in the caller context
        for (name, &id) in &self.var_ids {
            runtime_ctx.declare_var(name, id);
        }

        // Return all variables to the caller context
        self.execution.return_to_caller(runtime_ctx);

        Ok(())
    }

    /// Assigns a variable identifier and declares it in the execution context.
    pub fn declare_var(&mut self, name: &str, id: u32) -> u32 {
        // Set the variable id
        self.var_ids.insert(name.to_string(), id);
        debug!("[RuntimeContext] {} is now with id {}", name, id);

        // Declare the variable in the execution context
        self.execution.declare_var(name);

        id
    }

    /// Assigns a value to a variable in the execution context.
    /// If the variable is not declared, it will return an error.
    pub fn set_var(&mut self, name: &str, value: u32) -> Result<(), RuntimeError> {
        if !self.var_ids.contains_key(name) {
            return Err(RuntimeError::VariableNotDeclared);
        }

        self.execution.set_var(name, value)
    }

    /// Unsets a variable in the execution context.
    /// If the variable is not declared, it will return an error.
    pub fn unset_var(&mut self, name: &str) -> Result<(), RuntimeError> {
        if !self.var_ids.contains_key(name) {
            return Err(RuntimeError::VariableNotDeclared);
        }

        self.execution.unset_var(name)
    }

    /// Gets the value of a variable from the execution context.
    /// If the variable is not declared, it will return an error.
    pub fn get_value(&self, name: &str) -> Result<u32, RuntimeError> {
        if !self.var_ids.contains_key(name) {
            return Err(RuntimeError::VariableNotDeclared);
        }

        match self.execution.get_var(name)? {
            Some(value) => Ok(value),
            None => Ok(0),
        }
    }

    /// Gets the id of a variable in the runtime context.
    /// If the variable is not declared, it will return an error.
    pub fn get_var_id(&self, var_name: &str) -> Result<u32, RuntimeError> {
        self.var_ids
            .get(var_name)
            .copied()
            .ok_or(RuntimeError::VariableNotDeclared)
    }
}

/// Execution Context
/// Handles variable operations and values for a specific runtime context.
#[derive(Clone)]
pub struct ExecutionContext {
    caller_id: u32,
    context_id: u32,
    vars: HashMap<String, Option<u32>>, // Variable Name -> Variable Value
}

impl ExecutionContext {
    /// Constructs a new runtime execution context with specified caller and context IDs.
    pub fn new(caller_id: u32, context_id: u32) -> Self {
        Self {
            caller_id,
            context_id,
            vars: HashMap::new(),
        }
    }

    /// Clones all variables from the specified context into this context.
    pub fn load_context(&mut self, context: &RuntimeContext) -> &mut Self {
        self.vars = context.execution.vars.clone();
        self
    }

    /// Updates all variables from this context back to the caller's context.
    pub fn return_to_caller(&mut self, context: &mut RuntimeContext) -> Result<(), RuntimeError> {
        for (name, &val) in &self.vars {
            match val {
                Some(value) => context.execution.set_var(name, value)?,
                None => context.execution.unset_var(name)?,
            }
        }
        Ok(())
    }

    /// Declares a new variable in the context without setting its value (initialized as unset).
    /// If the variable is already declared, it will be overwritten.
    pub fn declare_var(&mut self, var_name: &str) {
        self.vars.insert(var_name.to_owned(), None);
        debug!("[ExecutionContext] '{}' is declared", var_name);
    }

    /// Retrieves the value of a variable if it is set, or None if it is unset.
    /// If the variable is not declared, it will return an error.
    pub fn get_var(&self, var_name: &str) -> Result<Option<u32>, RuntimeError> {
        match self.vars.get(var_name) {
            Some(&value) => Ok(value),
            None => Err(RuntimeError::VariableNotDeclared),
        }
    }

    /// Sets the value of a declared variable, marking it as set.
    /// If the variable is not declared, it will return an error.
    pub fn set_var(&mut self, var_name: &str, var_val: u32) -> Result<(), RuntimeError> {
        match self.vars.get_mut(var_name) {
            Some(value) => {
                *value = Some(var_val);
                debug!("[ExecutionContext] '{}' set to {}", var_name, var_val);
                Ok(())
            }
            None => Err(RuntimeError::VariableNotDeclared),
        }
    }

    /// Unsets (clears) a specified variable.
    /// If the variable is not declared, it will return an error.
    pub fn unset_var(&mut self, var_name: &str) -> Result<(), RuntimeError> {
        match self.vars.get_mut(var_name) {
            Some(value) => {
                *value = None;
                debug!("[ExecutionContext] '{}' is unset", var_name);
                Ok(())
            }
            None => Err(RuntimeError::VariableNotDeclared),
        }
    }
}

/// Runtime errors
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Error retrieving context")]
    ContextRetrievalError,
    #[error("Empty context stack")]
    EmptyContextStack,
    #[error("Variable is not declared")]
    VariableNotDeclared,
}
