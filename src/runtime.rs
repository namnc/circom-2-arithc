//! # Runtime Module
//!
//! This module manages the runtime environment for arithmetic circuit computation, handling variable and execution context tracking.

use std::collections::{HashMap, HashSet, LinkedList};

use log::debug;

/// Circom Runtime - the main runtime struct.
pub struct CircomRuntime {
    pub last_var_id: u32,
    pub last_context_id: u32,
    pub call_stack: LinkedList<RuntimeContext>,
}

impl CircomRuntime {
    pub fn new() -> CircomRuntime {
        CircomRuntime {
            last_var_id: 0,
            last_context_id: 0,
            call_stack: LinkedList::new(),
        }
    }

    pub fn init(&mut self) {
        self.last_context_id += 1;
        let rc = RuntimeContext::new(0, self.last_context_id);
        // When we init the circom runtime there is no caller to init with
        // rc.init(self);
        self.call_stack.push_front(rc);
    }

    // Should add context from calling and branching
    // Notice that while loop and block also has local variable so to consider as calling

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

    // pub fn get_runtime_context_by_context_id(&self, cid: u32) -> &RuntimeContext {
    //     let idx = self.get_runtime_context_index_in_stack_by_context_id(cid);
    //     self.get_runtime_context_by_stack_index(idx)
    // }

    pub fn get_current_runtime_context_caller_id(&self) -> u32 {
        self.call_stack.front().unwrap().caller_id
    }

    pub fn get_current_runtime_context_id(&mut self) -> u32 {
        self.call_stack.front().unwrap().context_id
    }

    // TODO
    // pub fn get_current_runtime_context_caller (&self) -> &RuntimeContext {
    //     let caller_id = self.get_current_runtime_context_caller_id();
    //     self.get_runtime_context_by_context_id(caller_id)

    // }

    pub fn get_current_runtime_context(&self) -> &RuntimeContext {
        self.call_stack.front().unwrap()
    }

    pub fn get_current_runtime_context_mut(&mut self) -> &mut RuntimeContext {
        self.call_stack.front_mut().unwrap()
    }

    pub fn get_var_from_current_context(&self, var: &String) -> u32 {
        let current = self.get_current_runtime_context();
        current.get_var(var)
    }
    pub fn assign_var_to_current_context(&mut self, var: &String) -> u32 {
        self.last_var_id += 1;
        let var_id = self.last_var_id;
        let current = self.get_current_runtime_context_mut();
        current.assign_var(var, var_id)
    }
    pub fn can_get_var_val_from_current_context(&self, var: &String) -> bool {
        let current = self.get_current_runtime_context();
        current.can_get_var_val(var)
    }
    pub fn get_var_val_from_current_context(&self, var: &String) -> u32 {
        let current = self.get_current_runtime_context();
        current.get_var_val(var)
    }
    pub fn assign_var_val_to_current_context(&mut self, var: &String, var_val: u32) -> u32 {
        let current = self.get_current_runtime_context_mut();
        current.assign_var_val(var, var_val)
    }

    pub fn deassign_var_val_to_current_context(&mut self, var: &String) -> u32 {
        let current = self.get_current_runtime_context_mut();
        current.deassign_var_val(var)
    }

    pub fn assign_auto_var_to_current_context(&mut self) -> String {
        self.last_var_id += 1;
        let var_id = self.last_var_id;
        let current = self.get_current_runtime_context_mut();
        let var = format!("auto_var_{}", var_id);
        current.assign_var(&var, var_id);
        println!("[CircomRuntime] Auto var {}", var);
        var
    }

    // TODO: array auto var should support multi-dimension, right now 1

    pub fn assign_array_var_to_current_context(
        &mut self,
        var: &String,
        indice: Vec<u32>,
    ) -> (String, u32) {
        self.last_var_id += 1;
        let var_id = self.last_var_id;
        let current = self.get_current_runtime_context_mut();
        let mut access_index = String::new();
        for i in 0..indice.len() {
            access_index.push_str(&format!("_{}", indice[i]));
        }
        let var = format!("{}{}", var, access_index);
        current.assign_var(&var, var_id);
        println!("[CircomRuntime] Array var {}", var);
        (var, var_id)
    }
}

/// Runtime Context - tracks the variable and execution context for a specific environment.
/// For runtime context we mostly care about the `var_id`.
/// For runtime execution we care about the evaluated value of a named variable.
pub struct RuntimeContext {
    pub caller_id: u32,
    pub context_id: u32,
    pub vars: HashMap<String, u32>,
    pub execution: RuntimeExecutionContext,
}

impl RuntimeContext {
    pub fn new(_caller_id: u32, _context_id: u32) -> RuntimeContext {
        RuntimeContext {
            caller_id: _caller_id,
            context_id: _context_id,
            vars: HashMap::new(),
            execution: RuntimeExecutionContext::new(_caller_id, _context_id),
        }
    }

    // pub fn init (&mut self, runtime: &CircomRuntime) {
    //     let context = runtime.get_runtime_context_by_context_id(self.caller_id);
    //     for (k, v) in context.vars.iter() {
    //         self.assign_var(k, *v);
    //     }
    //     self.execution.init(context);
    // }

    // Return to caller or return from callee to push changes from a function call back to caller
    // pub fn return_to_caller(&mut self, runtime: &CircomRuntime) {
    //     let context = runtime.get_runtime_context_by_context_id(self.caller_id);
    //     for (k, v) in self.vars.iter() {
    //         context.assign_var(k, *v);
    //     }
    //     self.execution.return_to_caller(context);
    // }

    pub fn assign_var(&mut self, var_name: &String, last_var_id: u32) -> u32 {
        self.vars.insert(var_name.to_string(), last_var_id);
        self.execution.assign_var(var_name);
        println!(
            "[RuntimeContext] {} is now with id {}",
            var_name, last_var_id
        );
        last_var_id
    }

    pub fn assign_var_val(&mut self, var_name: &String, var_val: u32) -> u32 {
        if !self.can_get_var(var_name) {
            return 0;
        }
        self.execution.assign_var_val(var_name, var_val);
        var_val
    }

    pub fn deassign_var_val(&mut self, var_name: &String) -> u32 {
        if !self.can_get_var(var_name) {
            return 0;
        }
        self.execution.deassign_var_val(var_name);
        0
    }

    pub fn can_get_var(&self, var_name: &String) -> bool {
        self.vars.contains_key(var_name)
    }

    pub fn get_var(&self, var_name: &String) -> u32 {
        if !self.can_get_var(var_name) {
            return 0;
        }
        *self.vars.get(var_name).unwrap()
    }

    pub fn can_get_var_val(&self, var_name: &String) -> bool {
        self.execution.can_get_var_val(var_name)
    }

    pub fn get_var_val(&self, var_name: &String) -> u32 {
        if !self.execution.can_get_var_val(var_name) {
            return 0;
        }
        self.execution.get_var_val(var_name)
    }
}

/// Runtime Execution Context
pub struct RuntimeExecutionContext {
    pub caller_id: u32,
    pub context_id: u32,
    pub vars: HashMap<String, Option<u32>>,
}

impl RuntimeExecutionContext {
    /// Constructs a new runtime execution context with specified caller and context IDs, cloning variables from the given context.
    pub fn new(caller_id: u32, context_id: u32, context: &RuntimeContext) -> Self {
        Self {
            caller_id,
            context_id,
            vars: context.execution.vars.clone(),
        }
    }

    /// Copies all variables from this context back to the caller's context.
    pub fn return_to_caller(&mut self, context: &mut RuntimeContext) {
        self.vars.iter().for_each(|(name, &val)| match val {
            Some(value) => context.execution.set_var(name, value),
            None => context.execution.unset_var(name),
        });
    }

    /// Declares a new variable in the context without setting its value (initialized as unset).
    pub fn declare_var(&mut self, var_name: &str) {
        self.vars.insert(var_name.to_owned(), None);
        debug!("[RuntimeExecutionContext] '{}' is declared", var_name);
    }

    /// Retrieves the value of a variable if it is set, or None if it is unset.
    pub fn get_var(&self, var_name: &str) -> Option<u32> {
        self.vars.get(var_name).cloned().flatten()
    }

    /// Sets the value of a specified variable, marking it as set.
    pub fn set_var(&mut self, var_name: &str, var_val: u32) {
        self.vars.insert(var_name.to_owned(), Some(var_val));
        debug!(
            "[RuntimeExecutionContext] '{}' set to {}",
            var_name, var_val
        );
    }

    /// Unsets (clears) a specified variable.
    pub fn unset_var(&mut self, var_name: &str) {
        if self.vars.contains_key(var_name) {
            self.vars.insert(var_name.to_owned(), None);
            debug!("[RuntimeExecutionContext] '{}' is unset", var_name);
        }
    }
}
