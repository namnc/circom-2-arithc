//! # Runtime Module
//!
//! This module manages the runtime environment for arithmetic circuit computation, handling variable and execution context tracking.

use std::collections::{HashMap, LinkedList};

/// Runtime Context struct.
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

/// Runtime Execution Context struct
pub struct RuntimeExecutionContext {
    pub caller_id: u32,
    pub context_id: u32,
    pub vars: HashMap<String, u32>,
    pub exevars: HashMap<String, bool>,
}

impl RuntimeExecutionContext {
    pub fn new(_caller_id: u32, _context_id: u32) -> RuntimeExecutionContext {
        RuntimeExecutionContext {
            caller_id: _caller_id,
            context_id: _context_id,
            vars: HashMap::new(),
            exevars: HashMap::new(),
        }
    }

    pub fn init(&mut self, context: &RuntimeContext) {
        for (k, v) in context.execution.vars.iter() {
            self.assign_var(k);
            if context.execution.can_get_var_val(k) {
                self.assign_var_val(k, *v);
            }
        }
    }

    // Return to caller or return from callee to push changes from a function call back to caller
    // pub fn return_to_caller(&mut self, context: &mut RuntimeContext) {
    //     for (k, v) in self.vars.iter() {
    //         context.execution.assign_var(k);
    //         if self.can_get_var_val(k) {
    //             context.execution.assign_var_val(k, *v);
    //         }
    //     }
    // }

    pub fn assign_var(&mut self, var_name: &String) -> u32 {
        let mut var_val = 0;
        if self.exevars.contains_key(var_name) {
            var_val = self.get_var_val(var_name);
            self.vars.insert(var_name.to_string(), var_val);
            println!(
                "[RuntimeExecutionContext] Now {} carries over val {}",
                var_name, var_val
            );
        } else {
            self.vars.insert(var_name.to_string(), 0);
            self.exevars.insert(var_name.to_string(), false);
            println!(
                "[RuntimeExecutionContext] Now {} has no val {}",
                var_name, var_val
            );
        }
        var_val
    }

    pub fn assign_var_val(&mut self, var_name: &String, var_val: u32) -> u32 {
        self.vars.insert(var_name.to_string(), var_val);
        self.exevars.insert(var_name.to_string(), true);
        println!(
            "[RuntimeExecutionContext] Now {} has val {}",
            var_name, var_val
        );
        var_val
    }

    pub fn deassign_var_val(&mut self, var_name: &String) -> u32 {
        self.vars.insert(var_name.to_string(), 0);
        self.exevars.insert(var_name.to_string(), false);
        println!(
            "[RuntimeExecutionContext] Now {} has no val {}",
            var_name, 0
        );
        0
    }

    pub fn get_var_val(&self, var_name: &String) -> u32 {
        if !self.can_get_var_val(var_name) {
            return 0;
        }
        *self.vars.get(var_name).unwrap()
    }

    pub fn can_get_var_val(&self, var_name: &String) -> bool {
        if !self.exevars.contains_key(var_name) {
            return false;
        }
        *self.exevars.get(var_name).unwrap()
    }
}

/// For runtime we maintain a call stack
/// Right now this is buggy cannot handle the stack management (RUST!!!)
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
