//! # Runtime Module
//!
//! This module manages the main runtime, keeping track of the multiple contexts and data items in the program.

use crate::program::ProgramError;
use circom_program_structure::ast::VariableType;
use rand::{thread_rng, Rng};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    fmt::Write,
    rc::Rc,
};
use thiserror::Error;

pub const RETURN_VAR: &str = "function_return_value";

/// Data type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataType {
    Component,
    Signal,
    Variable,
}

impl TryFrom<&VariableType> for DataType {
    type Error = RuntimeError;
    fn try_from(t: &VariableType) -> Result<Self, Self::Error> {
        match t {
            VariableType::Component => Ok(DataType::Component),
            VariableType::Signal(..) => Ok(DataType::Signal),
            VariableType::Var => Ok(DataType::Variable),
            _ => Err(RuntimeError::UnsupportedDataType),
        }
    }
}

/// Structure to hold either a single or a nested array of values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NestedValue<T> {
    Array(Vec<NestedValue<T>>),
    Value(T),
}

/// Data item sub access.
/// - The component property is used to access component signals (by name).
/// - The array property is used to access an array index.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SubAccess {
    Array(u32),
    Component(String),
}

/// Manages a stack of execution contexts for a runtime environment.
#[derive(Debug, Default)]
pub struct Runtime {
    contexts: VecDeque<Context>,
    next_signal_id: Rc<RefCell<u32>>,
}

impl Runtime {
    /// Creates an empty runtime with no contexts.
    pub fn new() -> Self {
        Self {
            contexts: VecDeque::from([Context::new("0".to_string())]),
            next_signal_id: Rc::new(RefCell::new(0)),
        }
    }

    /// Adds a new context onto the stack, optionally inheriting from the current context.
    pub fn push_context(&mut self, inherit: bool, id: String) -> Result<(), RuntimeError> {
        let new_context = if inherit {
            match self.contexts.front() {
                Some(parent_context) => Context::new_with_inheritance(parent_context),
                None => return Err(RuntimeError::NoContextToInheritFrom),
            }
        } else {
            Context::new(id)
        };
        self.contexts.push_front(new_context);
        Ok(())
    }

    /// Removes the current context from the stack, with an option to merge it into its parent.
    pub fn pop_context(&mut self, merge: bool) -> Result<(), RuntimeError> {
        if self.contexts.is_empty() {
            return Err(RuntimeError::EmptyContextStack);
        }

        if merge && self.contexts.len() > 1 {
            let child_context = self
                .contexts
                .pop_front()
                .ok_or(RuntimeError::ContextRetrievalError)?;
            let parent_context = self
                .contexts
                .front_mut()
                .ok_or(RuntimeError::ContextRetrievalError)?;
            parent_context.merge(&child_context)?;
        } else {
            self.contexts.pop_front();
        }

        Ok(())
    }

    /// Returns a mutable reference to the current context.
    pub fn current_context(&mut self) -> Result<&mut Context, RuntimeError> {
        self.contexts
            .front_mut()
            .ok_or(RuntimeError::EmptyContextStack)
    }

    /// Returns a clone of the Rc<RefCell<u32>> for next_signal_id.
    pub fn get_signal_gen(&self) -> Rc<RefCell<u32>> {
        Rc::clone(&self.next_signal_id)
    }

    /// Generates a new unique signal ID.
    fn gen_signal(next_signal_id: Rc<RefCell<u32>>) -> u32 {
        let mut id_ref = next_signal_id.borrow_mut();
        let id = *id_ref;
        *id_ref += 1;
        id
    }
}

/// Holds the state of a single execution context or scope.
#[derive(Debug, Clone)]
pub struct Context {
    ctx_name: String,
    names: HashSet<String>,
    variables: HashMap<String, Variable>,
    signals: HashMap<String, Signal>,
    components: HashMap<String, Component>,
}

impl Context {
    /// Constructs a new Context.
    pub fn new(ctx_name: String) -> Self {
        Self {
            ctx_name,
            names: HashSet::new(),
            variables: HashMap::new(),
            signals: HashMap::new(),
            components: HashMap::new(),
        }
    }

    /// Returns a contexts that inherits from the current context.
    pub fn new_with_inheritance(&self) -> Self {
        Self {
            ctx_name: self.ctx_name.clone(),
            names: self.names.clone(),
            variables: self.variables.clone(),
            signals: self.signals.clone(),
            components: self.components.clone(),
        }
    }

    pub fn get_ctx_name(&self) -> String {
        self.ctx_name.clone()
    }

    /// Merges changes from the given context into this context.
    /// Signals are not merged, as they are read-only.
    pub fn merge(&mut self, child: &Context) -> Result<(), RuntimeError> {
        for (name, variable) in &child.variables {
            if self.variables.contains_key(name) {
                self.variables.insert(name.clone(), variable.clone());
            }
        }

        // Force the merge of the return variable.
        if child.variables.contains_key(RETURN_VAR) {
            self.variables
                .insert(RETURN_VAR.to_string(), child.variables[RETURN_VAR].clone());
        }

        for (name, component) in &child.components {
            if self.components.contains_key(name) {
                self.components.insert(name.clone(), component.clone());
            }
        }

        Ok(())
    }

    /// Declares a new item of the specified type with the given name and dimensions.
    pub fn declare_item(
        &mut self,
        data_type: DataType,
        name: &str,
        dimensions: &[u32],
        next_signal_id: Rc<RefCell<u32>>,
    ) -> Result<(), RuntimeError> {
        // Parse name
        let name = name.to_string();

        // Check availability. Ignore variables redeclaration.
        if !self.names.insert(name.clone()) && data_type != DataType::Variable {
            return Err(RuntimeError::ItemAlreadyDeclared);
        }

        match data_type {
            DataType::Signal => {
                let signal = Signal::new(dimensions, next_signal_id);
                self.signals.insert(name, signal);
            }
            DataType::Variable => {
                let variable = Variable::new(dimensions);
                self.variables.insert(name, variable);
            }
            DataType::Component => {
                let component = Component::new(dimensions);
                self.components.insert(name, component);
            }
        };

        Ok(())
    }

    /// Declares a new item with a random name.
    pub fn declare_random_item(
        &mut self,
        next_signal_id: Rc<RefCell<u32>>,
        data_type: DataType,
    ) -> Result<DataAccess, RuntimeError> {
        let name = format!("random_{}", generate_u32());
        self.declare_item(data_type, &name, &[], next_signal_id)?;
        Ok(DataAccess::new(&name, vec![]))
    }

    /// Returns the data type of an item.
    pub fn get_item_data_type(&self, name: &str) -> Result<DataType, RuntimeError> {
        if self.variables.contains_key(name) {
            Ok(DataType::Variable)
        } else if self.signals.contains_key(name) {
            Ok(DataType::Signal)
        } else if self.components.contains_key(name) {
            Ok(DataType::Component)
        } else {
            Err(RuntimeError::ItemNotDeclared(format!(
                "get_item_data_type: {}",
                name
            )))
        }
    }

    /// Sets the content of a variable.
    pub fn set_variable(
        &mut self,
        access: &DataAccess,
        value: Option<u32>,
    ) -> Result<(), RuntimeError> {
        let variable =
            self.variables
                .get_mut(&access.name)
                .ok_or(RuntimeError::ItemNotDeclared(format!(
                    "set_variable: {:?}",
                    access
                )))?;

        variable.set(&access_to_u32(access.get_access())?, value)
    }

    /// Gets a variable whole content.
    pub fn get_variable(&self, name: &str) -> Result<Variable, RuntimeError> {
        self.variables
            .get(name)
            .ok_or(RuntimeError::ItemNotDeclared(format!(
                "get_variable: {}",
                name
            )))
            .cloned()
    }

    /// Gets a variable single or nested content.
    pub fn get_variable_content(
        &self,
        access: &DataAccess,
    ) -> Result<NestedValue<Option<u32>>, RuntimeError> {
        let variable = self
            .variables
            .get(&access.name)
            .ok_or(RuntimeError::ItemNotDeclared(format!(
                "get_variable: {:?}",
                access
            )))?;

        variable.get(&access_to_u32(access.get_access())?)
    }

    /// Gets the content of a variable.
    pub fn get_variable_value(&self, access: &DataAccess) -> Result<Option<u32>, RuntimeError> {
        let variable = self
            .variables
            .get(&access.name)
            .ok_or(RuntimeError::ItemNotDeclared(format!(
                "get_variable_value: {:?}",
                access
            )))?;

        variable.get_value(&access_to_u32(access.get_access())?)
    }

    /// Gets a signal with all its dimensions.
    pub fn get_signal(&self, name: &str) -> Result<Signal, RuntimeError> {
        self.signals
            .get(name)
            .ok_or(RuntimeError::ItemNotDeclared(format!(
                "get_signal: {}",
                name
            )))
            .cloned()
    }

    /// Gets a signal content at the specified index path.
    pub fn get_signal_content(
        &self,
        access: &DataAccess,
    ) -> Result<NestedValue<u32>, RuntimeError> {
        let signal = self
            .signals
            .get(&access.name)
            .ok_or(RuntimeError::ItemNotDeclared(format!(
                "get_signal_content: {:?}",
                access
            )))?;

        signal.get(&access_to_u32(access.get_access())?)
    }

    /// Gets the id of the signal at the specified index path.
    /// This will return an error if the index path doesn't point to a single value.
    pub fn get_signal_id(&self, access: &DataAccess) -> Result<u32, RuntimeError> {
        let signal = self
            .signals
            .get(&access.name)
            .ok_or(RuntimeError::ItemNotDeclared(format!(
                "get_signal_id: {:?}",
                access
            )))?;

        match signal.get(&access_to_u32(access.get_access())?)? {
            NestedValue::Value(id) => Ok(id),
            NestedValue::Array(_) => Err(RuntimeError::NotAValue),
        }
    }

    /// Gets a component.
    pub fn get_component_map(
        &self,
        access: &DataAccess,
    ) -> Result<HashMap<String, Signal>, RuntimeError> {
        let component = self
            .components
            .get(&access.name)
            .ok_or(RuntimeError::ItemNotDeclared(format!(
                "get_component_map: {:?}",
                access
            )))?;

        component.get_map(&access_to_u32(access.get_access())?)
    }

    /// Gets the id of a component's signal.
    pub fn get_component_signal_id(&self, access: &DataAccess) -> Result<u32, RuntimeError> {
        let (component_access, signal_access) = process_component_access(access)?;
        let component =
            self.components
                .get(&component_access.name)
                .ok_or(RuntimeError::ItemNotDeclared(format!(
                    "get_component_signal_id: {:?}",
                    access
                )))?;

        component.get_signal_id(
            &access_to_u32(component_access.get_access())?,
            &signal_access,
        )
    }

    /// Gets the content of a component's signal.
    pub fn get_component_signal_content(
        &self,
        access: &DataAccess,
    ) -> Result<NestedValue<u32>, RuntimeError> {
        let (component_access, signal_access) = process_component_access(access)?;
        let component =
            self.components
                .get(&component_access.name)
                .ok_or(RuntimeError::ItemNotDeclared(format!(
                    "get_component_signal_id: {:?}",
                    access
                )))?;

        component.get_signal_content(
            &access_to_u32(component_access.get_access())?,
            &signal_access,
        )
    }

    /// Sets a component's input/output signal map.
    pub fn set_component(
        &mut self,
        access: &DataAccess,
        map: HashMap<String, Signal>,
    ) -> Result<(), RuntimeError> {
        let component =
            self.components
                .get_mut(&access.name)
                .ok_or(RuntimeError::ItemNotDeclared(format!(
                    "set_component: {:?}",
                    access
                )))?;

        component.set_signal_map(&access_to_u32(access.get_access())?, map)
    }
}

/// Represents a signal that holds a single id or a nested structure of values with unique IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signal {
    value: NestedValue<u32>,
}

impl Signal {
    /// Constructs a new Signal as a nested structure based on provided dimensions.
    fn new(dimensions: &[u32], next_signal_id: Rc<RefCell<u32>>) -> Self {
        fn create_nested_signal(
            dimensions: &[u32],
            next_signal_id: Rc<RefCell<u32>>,
        ) -> NestedValue<u32> {
            if let Some((&first, rest)) = dimensions.split_first() {
                let array = (0..first)
                    .map(|_| create_nested_signal(rest, next_signal_id.clone()))
                    .collect();
                NestedValue::Array(array)
            } else {
                // Generate a new signal ID
                let id = Runtime::gen_signal(next_signal_id);
                NestedValue::Value(id)
            }
        }

        Self {
            value: create_nested_signal(dimensions, next_signal_id),
        }
    }

    /// Retrieves the nested value at the specified index path.
    fn get(&self, index_path: &[u32]) -> Result<NestedValue<u32>, RuntimeError> {
        get_nested_value(&self.value, index_path)
    }

    // Retrieves the id of the signal at the specified index path.
    fn get_id(&self, index_path: &[u32]) -> Result<u32, RuntimeError> {
        match self.get(index_path)? {
            NestedValue::Value(id) => Ok(id),
            NestedValue::Array(_) => Err(RuntimeError::NotAValue),
        }
    }
}

/// Represents a variable that can hold a single value or nested structure of values.
#[derive(Clone, Debug)]
pub struct Variable {
    value: NestedValue<Option<u32>>,
}

impl Variable {
    /// Constructs a new Variable as a nested structure based on provided dimensions.
    fn new(dimensions: &[u32]) -> Self {
        // Initialize the innermost value.
        let mut value = NestedValue::Value(None);

        // Construct the nested structure in reverse order to ensure the correct dimensionality.
        for &dimension in dimensions.iter().rev() {
            let array = vec![value.clone(); dimension as usize];
            value = NestedValue::Array(array);
        }

        Self { value }
    }

    /// Sets the content of the variable at the specified index path.
    fn set(&mut self, index_path: &[u32], val: Option<u32>) -> Result<(), RuntimeError> {
        let inner_value = get_mut_nested_value(&mut self.value, index_path)?;

        match inner_value {
            NestedValue::Array(_) => Err(RuntimeError::NotAValue),
            NestedValue::Value(_) => {
                *inner_value = NestedValue::Value(val);
                Ok(())
            }
        }
    }

    /// Retrieves the content of the variable at the specified index path.
    fn get(&self, index_path: &[u32]) -> Result<NestedValue<Option<u32>>, RuntimeError> {
        get_nested_value(&self.value, index_path)
    }

    /// Retrieves the value of the variable at the specified index path.
    fn get_value(&self, index_path: &[u32]) -> Result<Option<u32>, RuntimeError> {
        match self.get(index_path)? {
            NestedValue::Value(val) => Ok(val),
            NestedValue::Array(_) => Err(RuntimeError::NotAValue),
        }
    }
}

/// Stores a component's input/output signals with their respective identifiers.
#[derive(Clone, Debug)]
pub struct Component {
    signal_map: NestedValue<HashMap<String, Signal>>,
}

impl Component {
    /// Constructs a new Component as a nested structure based on provided dimensions.
    fn new(dimensions: &[u32]) -> Self {
        let mut signal_map = NestedValue::Value(HashMap::new());

        // Construct the nested structure in reverse order to ensure the correct dimensionality.
        for &dimension in dimensions.iter().rev() {
            let array = vec![signal_map.clone(); dimension as usize];
            signal_map = NestedValue::Array(array);
        }

        Self { signal_map }
    }

    /// Retrieves the component signal map at the specified index path.
    fn get_map(&self, index_path: &[u32]) -> Result<HashMap<String, Signal>, RuntimeError> {
        let nested_val = get_nested_value(&self.signal_map, index_path)?;

        match nested_val {
            NestedValue::Value(map) => Ok(map),
            NestedValue::Array(_) => Err(RuntimeError::NotAValue),
        }
    }

    /// Sets the signal map
    fn set_signal_map(
        &mut self,
        component_access: &[u32],
        map: HashMap<String, Signal>,
    ) -> Result<(), RuntimeError> {
        let nested_val = get_mut_nested_value(&mut self.signal_map, component_access)?;

        let nested_map = match nested_val {
            NestedValue::Value(map) => map,
            NestedValue::Array(_) => return Err(RuntimeError::NotAValue),
        };

        *nested_map = map;

        Ok(())
    }

    /// Returns the signal's content at the specified index path.
    fn get_signal_content(
        &self,
        component_access: &[u32],
        signal_access: &DataAccess,
    ) -> Result<NestedValue<u32>, RuntimeError> {
        let nested_val = get_nested_value(&self.signal_map, component_access)?;

        let map = match nested_val {
            NestedValue::Value(map) => map,
            NestedValue::Array(_) => return Err(RuntimeError::NotAValue),
        };

        let signal = map
            .get(&signal_access.get_name())
            .ok_or(RuntimeError::ItemNotDeclared(format!(
                "get_signal_id: {:?}",
                signal_access
            )))?;

        signal.get(&access_to_u32(signal_access.get_access())?)
    }

    /// Returns the signal's ID at the specified index path.
    fn get_signal_id(
        &self,
        component_access: &[u32],
        signal_access: &DataAccess,
    ) -> Result<u32, RuntimeError> {
        let nested_val = get_nested_value(&self.signal_map, component_access)?;

        let map = match nested_val {
            NestedValue::Value(map) => map,
            NestedValue::Array(_) => return Err(RuntimeError::NotAValue),
        };

        let signal = map
            .get(&signal_access.get_name())
            .ok_or(RuntimeError::ItemNotDeclared(format!(
                "get_signal_id: {:?}",
                signal_access
            )))?;

        signal.get_id(&access_to_u32(signal_access.get_access())?)
    }
}

/// Data Access structure.
/// - The name property is used to access variables, signals and components (by name).
/// - The access property is used to access an array index or a component signal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataAccess {
    name: String,
    access: Vec<SubAccess>,
}

impl DataAccess {
    /// Constructs a new DataAccess.
    pub fn new(name: &str, access: Vec<SubAccess>) -> Self {
        Self {
            name: name.to_string(),
            access,
        }
    }

    /// Sets the name of the data item.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Gets the name of the data item.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Sets the sub access of the data item.
    pub fn set_access(&mut self, access: Vec<SubAccess>) {
        self.access = access;
    }

    /// Gets the sub access of the data item.
    pub fn get_access(&self) -> &Vec<SubAccess> {
        &self.access
    }

    /// Gets the access string for labeling of the data item.
    pub fn access_str(&self, ctx_name: String) -> String {
        let mut ret = format!("{}.", ctx_name);
        ret.write_str(self.get_name().as_str()).ok();
        for sub in self.get_access() {
            match sub {
                SubAccess::Array(index) => {
                    ret.write_str(format!("[{}]", index).as_str()).ok();
                }
                SubAccess::Component(name) => {
                    ret.write_str(format!(".{}", name).as_str()).ok();
                }
            }
        }
        ret
    }
}

/// Processes an access to a component's signal.
/// Returns a tuple containing the component access, and the signal access.
/// (component_access, signal_access)
pub fn process_component_access(
    access: &DataAccess,
) -> Result<(DataAccess, DataAccess), RuntimeError> {
    let mut initial_path = Vec::new();
    let mut final_path = Vec::new();
    let mut signal_name = String::new();
    let mut has_signal = false;

    for sub_access in access.get_access() {
        match sub_access {
            SubAccess::Array(index) => {
                if has_signal {
                    final_path.push(*index);
                } else {
                    initial_path.push(*index);
                }
            }
            SubAccess::Component(name) => {
                if has_signal {
                    // We shouldn't have more than one signal in a sub access.
                    return Err(RuntimeError::AccessError);
                }
                signal_name.clone_from(name);
                has_signal = true;
            }
        }
    }

    if !has_signal {
        return Err(RuntimeError::AccessError);
    }

    Ok((
        DataAccess::new(&access.get_name(), u32_to_access(&initial_path)),
        DataAccess::new(&signal_name, u32_to_access(&final_path)),
    ))
}

/// Generic function to navigate through NestedValue and return the inner value.
/// The clone could be removed but then the type would need to implement Copy. Leaving it for now.
pub fn get_nested_value<T: Clone>(
    nested_value: &NestedValue<T>,
    index_path: &[u32],
) -> Result<NestedValue<T>, RuntimeError> {
    let mut current_level = nested_value;

    for &index in index_path {
        match current_level {
            NestedValue::Array(values) => {
                current_level = values
                    .get(index as usize)
                    .ok_or(RuntimeError::IndexOutOfBounds)?;
            }
            _ => return Err(RuntimeError::AccessError),
        }
    }

    Ok(current_level.clone())
}

/// Generic function to navigate through NestedValue and return a mutable reference to the inner value.
pub fn get_mut_nested_value<'a, T>(
    nested_value: &'a mut NestedValue<T>,
    index_path: &[u32],
) -> Result<&'a mut NestedValue<T>, RuntimeError> {
    let mut current_level = nested_value;
    for &index in index_path {
        current_level = match current_level {
            NestedValue::Array(values) => values
                .get_mut(index as usize)
                .ok_or(RuntimeError::IndexOutOfBounds)?,
            _ => return Err(RuntimeError::AccessError),
        };
    }

    Ok(current_level)
}

/// Converts a vector of u32 to a vector of SubAccess.
pub fn u32_to_access(indices: &[u32]) -> Vec<SubAccess> {
    indices
        .iter()
        .map(|&index| SubAccess::Array(index))
        .collect()
}

/// Converts a vector of SubAccess to a vector of u32.
pub fn access_to_u32(sub_accesses: &[SubAccess]) -> Result<Vec<u32>, RuntimeError> {
    sub_accesses
        .iter()
        .map(|sub_access| match sub_access {
            SubAccess::Array(index) => Ok(*index),
            _ => Err(RuntimeError::AccessError),
        })
        .collect()
}

/// Increments a multi-dimensional array index.
/// Returns a boolean that indicates if there are more elements to traverse.
///
/// * `indices` - A vector representing the current position in a multi-dimensional array.
/// * `limits` - A vector representing the limits of each dimension of the array.
pub fn increment_indices(indices: &mut [u32], limits: &[u32]) -> Result<bool, RuntimeError> {
    if indices.len() != limits.len() {
        return Err(RuntimeError::AccessError);
    }

    let mut carry = true;
    for (index, &limit) in indices.iter_mut().zip(limits.iter()).rev() {
        if carry {
            if *index < limit - 1 {
                *index += 1;
                carry = false;
            } else {
                *index = 0;
            }
        }
    }

    Ok(!carry)
}

/// Generates a random u32.
pub fn generate_u32() -> u32 {
    thread_rng().gen()
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Access Error")]
    AccessError,
    #[error("Error retrieving context")]
    ContextRetrievalError,
    #[error("Empty context stack")]
    EmptyContextStack,
    #[error("Index out of bounds")]
    IndexOutOfBounds,
    #[error("Item already declared")]
    ItemAlreadyDeclared,
    #[error("Item not declared: {0}")]
    ItemNotDeclared(String),
    #[error("No context to inherit from")]
    NoContextToInheritFrom,
    #[error("Data Item content is not a single value")]
    NotAValue,
    #[error("Unsupported data type")]
    UnsupportedDataType,
    #[error("Assertion failed")]
    AssertionFailed,
}

impl From<RuntimeError> for ProgramError {
    fn from(e: RuntimeError) -> Self {
        ProgramError::RuntimeError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_push_pop_context() {
        let mut runtime = Runtime::new();

        runtime.push_context(false, "ctx1".to_string()).unwrap();
        assert_eq!(runtime.contexts.front().unwrap().get_ctx_name(), "ctx1");

        runtime.push_context(false, "ctx2".to_string()).unwrap();
        assert_eq!(runtime.contexts.front().unwrap().get_ctx_name(), "ctx2");

        runtime.pop_context(false).unwrap();
        assert_eq!(runtime.contexts.front().unwrap().get_ctx_name(), "ctx1");

        runtime.pop_context(false).unwrap();
        assert_eq!(runtime.contexts.front().unwrap().get_ctx_name(), "0");

        runtime.pop_context(false).unwrap();
        let result = runtime.pop_context(false);
        assert!(result.is_err());
        if let Err(RuntimeError::EmptyContextStack) = result {
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_runtime_push_pop_context_with_inheritance() {
        let mut runtime = Runtime::new();

        runtime.push_context(false, "ctx1".to_string()).unwrap();
        {
            let next_signal_id = runtime.get_signal_gen();
            runtime
                .current_context()
                .unwrap()
                .declare_item(DataType::Variable, "var1", &[2, 2], next_signal_id.clone())
                .unwrap();
            runtime
                .current_context()
                .unwrap()
                .set_variable(
                    &DataAccess::new("var1", vec![SubAccess::Array(0), SubAccess::Array(0)]),
                    Some(42),
                )
                .unwrap();
        }

        runtime.push_context(true, "ctx2".to_string()).unwrap();
        {
            let var1_value = runtime
                .current_context()
                .unwrap()
                .get_variable_value(&DataAccess::new(
                    "var1",
                    vec![SubAccess::Array(0), SubAccess::Array(0)],
                ))
                .unwrap();
            assert_eq!(var1_value, Some(42));
        }

        runtime.pop_context(true).unwrap();
        {
            let var1_value = runtime
                .current_context()
                .unwrap()
                .get_variable_value(&DataAccess::new(
                    "var1",
                    vec![SubAccess::Array(0), SubAccess::Array(0)],
                ))
                .unwrap();
            assert_eq!(var1_value, Some(42));
        }
    }

    #[test]
    fn test_runtime_context_merge() {
        let mut runtime = Runtime::new();

        runtime.push_context(false, "ctx1".to_string()).unwrap();
        {
            let next_signal_id = runtime.get_signal_gen();
            runtime
                .current_context()
                .unwrap()
                .declare_item(DataType::Variable, "var1", &[2, 2], next_signal_id.clone())
                .unwrap();
            runtime
                .current_context()
                .unwrap()
                .set_variable(
                    &DataAccess::new("var1", vec![SubAccess::Array(0), SubAccess::Array(0)]),
                    Some(42),
                )
                .unwrap();
        }

        runtime.push_context(true, "ctx2".to_string()).unwrap();
        {
            runtime
                .current_context()
                .unwrap()
                .set_variable(
                    &DataAccess::new("var1", vec![SubAccess::Array(0), SubAccess::Array(0)]),
                    Some(100),
                )
                .unwrap();
        }

        runtime.pop_context(true).unwrap();
        {
            let var1_value = runtime
                .current_context()
                .unwrap()
                .get_variable_value(&DataAccess::new(
                    "var1",
                    vec![SubAccess::Array(0), SubAccess::Array(0)],
                ))
                .unwrap();
            assert_eq!(var1_value, Some(100));
        }
    }

    #[test]
    fn test_runtime_generate_signal_id() {
        let runtime = Runtime::new();
        let signal_id1 = Runtime::gen_signal(runtime.get_signal_gen());
        let signal_id2 = Runtime::gen_signal(runtime.get_signal_gen());
        assert_eq!(signal_id1, 0);
        assert_eq!(signal_id2, 1);
    }

    #[test]
    fn test_context_declare_item() {
        let mut context = Context::new("ctx1".to_string());
        let next_signal_id = Rc::new(RefCell::new(0));

        context
            .declare_item(DataType::Signal, "signal1", &[2, 2], next_signal_id.clone())
            .unwrap();
        context
            .declare_item(DataType::Variable, "var1", &[2, 2], next_signal_id.clone())
            .unwrap();
        context
            .declare_item(
                DataType::Component,
                "component1",
                &[2],
                next_signal_id.clone(),
            )
            .unwrap();

        assert!(context.signals.contains_key("signal1"));
        assert!(context.variables.contains_key("var1"));
        assert!(context.components.contains_key("component1"));

        let result = context.declare_item(DataType::Signal, "signal1", &[2, 2], next_signal_id);
        assert!(result.is_err());
        if let Err(RuntimeError::ItemAlreadyDeclared) = result {
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_context_set_get_variable() {
        let mut context = Context::new("ctx1".to_string());
        let next_signal_id = Rc::new(RefCell::new(0));
        context
            .declare_item(DataType::Variable, "var1", &[2, 2], next_signal_id)
            .unwrap();

        let access = DataAccess::new("var1", vec![SubAccess::Array(0), SubAccess::Array(0)]);
        context.set_variable(&access, Some(42)).unwrap();

        let value = context.get_variable_value(&access).unwrap();
        assert_eq!(value, Some(42));

        let content = context.get_variable_content(&access).unwrap();
        assert_eq!(content, NestedValue::Value(Some(42)));
    }

    #[test]
    fn test_context_get_signal() {
        let mut context = Context::new("ctx1".to_string());
        let next_signal_id = Rc::new(RefCell::new(0));
        context
            .declare_item(DataType::Signal, "signal1", &[2, 2], next_signal_id.clone())
            .unwrap();

        let signal = context.get_signal("signal1").unwrap();
        assert_eq!(signal.get(&[0, 0]).unwrap(), NestedValue::Value(0));
        assert_eq!(signal.get(&[1, 1]).unwrap(), NestedValue::Value(3));

        let result = context.get_signal("nonexistent_signal");
        assert!(result.is_err());
        if let Err(RuntimeError::ItemNotDeclared(_)) = result {
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_context_get_component_map() {
        let mut context = Context::new("ctx1".to_string());
        let next_signal_id = Rc::new(RefCell::new(0));
        context
            .declare_item(
                DataType::Component,
                "component1",
                &[2],
                next_signal_id.clone(),
            )
            .unwrap();

        let mut signal_map = HashMap::new();
        let signal = Signal::new(&[], next_signal_id.clone());
        signal_map.insert("signal1".to_string(), signal);

        let access = DataAccess::new("component1", vec![SubAccess::Array(0)]);
        context.set_component(&access, signal_map.clone()).unwrap();

        let retrieved_map = context.get_component_map(&access).unwrap();
        assert_eq!(retrieved_map, signal_map);

        let result = context.get_component_map(&DataAccess::new("nonexistent_component", vec![]));
        assert!(result.is_err());
        if let Err(RuntimeError::ItemNotDeclared(_)) = result {
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_context_merge() {
        let mut parent_context = Context::new("parent".to_string());
        let next_signal_id = Rc::new(RefCell::new(0));

        parent_context
            .declare_item(DataType::Variable, "var1", &[2, 2], next_signal_id.clone())
            .unwrap();
        parent_context
            .declare_item(DataType::Component, "comp1", &[1], next_signal_id.clone())
            .unwrap();

        let access_var1 = DataAccess::new("var1", vec![SubAccess::Array(0), SubAccess::Array(0)]);
        parent_context.set_variable(&access_var1, Some(42)).unwrap();

        let mut child_context = parent_context.new_with_inheritance();

        child_context.set_variable(&access_var1, Some(100)).unwrap();

        child_context
            .declare_item(DataType::Variable, "var2", &[1], next_signal_id.clone())
            .unwrap();
        let access_var2 = DataAccess::new("var2", vec![SubAccess::Array(0)]);
        child_context.set_variable(&access_var2, Some(7)).unwrap();

        parent_context.merge(&child_context).unwrap();

        assert_eq!(
            parent_context.get_variable_value(&access_var1).unwrap(),
            Some(100)
        );

        let result = parent_context.get_variable_value(&access_var2);
        assert!(result.is_err());
        if let Err(RuntimeError::ItemNotDeclared(_)) = result {
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_context_merge_with_return() {
        let mut parent_context = Context::new("parent".to_string());
        let next_signal_id = Rc::new(RefCell::new(0));

        parent_context
            .declare_item(DataType::Variable, "var1", &[2, 2], next_signal_id.clone())
            .unwrap();

        let access_var1 = DataAccess::new("var1", vec![SubAccess::Array(0), SubAccess::Array(0)]);
        parent_context.set_variable(&access_var1, Some(42)).unwrap();

        let mut child_context = parent_context.new_with_inheritance();

        // Declare and set the return variable in the child context
        child_context
            .declare_item(DataType::Variable, RETURN_VAR, &[1], next_signal_id.clone())
            .unwrap();
        let access_return_var = DataAccess::new(RETURN_VAR, vec![SubAccess::Array(0)]);
        child_context
            .set_variable(&access_return_var, Some(777))
            .unwrap();

        child_context.set_variable(&access_var1, Some(100)).unwrap();

        parent_context.merge(&child_context).unwrap();

        assert_eq!(
            parent_context.get_variable_value(&access_var1).unwrap(),
            Some(100)
        );
        assert_eq!(
            parent_context
                .get_variable_value(&access_return_var)
                .unwrap(),
            Some(777)
        );
    }

    #[test]
    fn test_context_errors() {
        let mut runtime = Runtime::new();
        runtime.pop_context(false).unwrap(); // Initial pop to empty stack

        let result = runtime.pop_context(false);
        assert!(result.is_err());
        if let Err(RuntimeError::EmptyContextStack) = result {
        } else {
            panic!("Unexpected error type");
        }

        let mut context = Context::new("ctx1".to_string());
        let next_signal_id = Rc::new(RefCell::new(0));
        context
            .declare_item(DataType::Variable, "var1", &[2, 2], next_signal_id.clone())
            .unwrap();

        let access = DataAccess::new(
            "nonexistent_var",
            vec![SubAccess::Array(0), SubAccess::Array(0)],
        );
        let result = context.get_variable_value(&access);
        assert!(result.is_err());
        if let Err(RuntimeError::ItemNotDeclared(_)) = result {
        } else {
            panic!("Unexpected error type");
        }

        let result = context.set_variable(&access, Some(42));
        assert!(result.is_err());
        if let Err(RuntimeError::ItemNotDeclared(_)) = result {
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_signal_new() {
        let signal = Signal::new(&[2, 3], Rc::new(RefCell::new(0)));
        if let NestedValue::Array(level1) = &signal.value {
            assert_eq!(level1.len(), 2);
            if let NestedValue::Array(level2) = &level1[0] {
                assert_eq!(level2.len(), 3);
                if let NestedValue::Value(id) = &level2[0] {
                    assert_eq!(*id, 0);
                } else {
                    panic!("Innermost value is not a Value");
                }
            } else {
                panic!("Second level is not an Array");
            }
        } else {
            panic!("First level is not an Array");
        }
    }

    #[test]
    fn test_signal_get() {
        let signal = Signal::new(&[2, 2], Rc::new(RefCell::new(0)));
        assert_eq!(signal.get(&[0, 0]).unwrap(), NestedValue::Value(0));
        assert_eq!(signal.get(&[1, 1]).unwrap(), NestedValue::Value(3));

        let result = signal.get(&[2, 0]);
        assert!(result.is_err());
        if let Err(RuntimeError::IndexOutOfBounds) = result {
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_signal_get_id() {
        let signal = Signal::new(&[2, 2], Rc::new(RefCell::new(0)));
        assert_eq!(signal.get_id(&[0, 0]).unwrap(), 0);
        assert_eq!(signal.get_id(&[1, 1]).unwrap(), 3);
        let result = signal.get_id(&[2, 0]);
        assert!(result.is_err());
        if let Err(RuntimeError::IndexOutOfBounds) = result {
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_variable_new() {
        let variable = Variable::new(&[2, 3]);
        if let NestedValue::Array(level1) = &variable.value {
            assert_eq!(level1.len(), 2);
            if let NestedValue::Array(level2) = &level1[0] {
                assert_eq!(level2.len(), 3);
                if let NestedValue::Value(value) = &level2[0] {
                    assert_eq!(*value, None);
                } else {
                    panic!("Innermost value is not a Value");
                }
            } else {
                panic!("Second level is not an Array");
            }
        } else {
            panic!("First level is not an Array");
        }
    }

    #[test]
    fn test_variable_set_get_error() {
        let mut variable = Variable::new(&[2, 2]);

        // Test setting a value out of bounds
        let result = variable.set(&[2, 0], Some(42));
        assert!(result.is_err());
        if let Err(RuntimeError::IndexOutOfBounds) = result {
        } else {
            panic!("Unexpected error type");
        }

        // Test getting a value out of bounds
        let result = variable.get(&[2, 0]);
        assert!(result.is_err());
        if let Err(RuntimeError::IndexOutOfBounds) = result {
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_component_new() {
        let component = Component::new(&[2, 3]);
        if let NestedValue::Array(level1) = &component.signal_map {
            assert_eq!(level1.len(), 2);
            if let NestedValue::Array(level2) = &level1[0] {
                assert_eq!(level2.len(), 3);
                if let NestedValue::Value(map) = &level2[0] {
                    assert_eq!(map.len(), 0);
                } else {
                    panic!("Innermost value is not a Value");
                }
            } else {
                panic!("Second level is not an Array");
            }
        } else {
            panic!("First level is not an Array");
        }
    }

    #[test]
    fn test_component_set_and_get_signal_map() {
        let mut component = Component::new(&[1]);
        let mut signal_map = HashMap::new();
        let signal = Signal::new(&[], Rc::new(RefCell::new(0)));
        signal_map.insert("signal1".to_string(), signal);

        component
            .set_signal_map(&[0], signal_map.clone())
            .expect("Setting signal map failed");

        let retrieved_map = component.get_map(&[0]).expect("Getting signal map failed");
        assert_eq!(retrieved_map, signal_map);
    }

    #[test]
    fn test_component_get_signal_content() {
        let mut component = Component::new(&[1]);
        let mut signal_map = HashMap::new();
        let signal = Signal::new(&[], Rc::new(RefCell::new(0)));
        signal_map.insert("signal1".to_string(), signal);

        component
            .set_signal_map(&[0], signal_map)
            .expect("Setting signal map failed");

        let access = DataAccess::new("signal1", vec![]);
        let content = component
            .get_signal_content(&[0], &access)
            .expect("Getting signal content failed");

        assert_eq!(content, NestedValue::Value(0));
    }

    #[test]
    fn test_component_get_signal_id() {
        let mut component = Component::new(&[1]);
        let mut signal_map = HashMap::new();
        let signal = Signal::new(&[], Rc::new(RefCell::new(0)));
        signal_map.insert("signal1".to_string(), signal);

        component
            .set_signal_map(&[0], signal_map)
            .expect("Setting signal map failed");

        let access = DataAccess::new("signal1", vec![]);
        let id = component
            .get_signal_id(&[0], &access)
            .expect("Getting signal ID failed");

        assert_eq!(id, 0);
    }

    #[test]
    fn test_component_nested_signal_map() {
        let mut component = Component::new(&[2]);
        let mut signal_map_0 = HashMap::new();
        let signal_0 = Signal::new(&[], Rc::new(RefCell::new(0)));
        signal_map_0.insert("signal1".to_string(), signal_0);

        component
            .set_signal_map(&[0], signal_map_0)
            .expect("Setting signal map failed");

        let mut signal_map_1 = HashMap::new();
        let signal_1 = Signal::new(&[], Rc::new(RefCell::new(1)));
        signal_map_1.insert("signal2".to_string(), signal_1);

        component
            .set_signal_map(&[1], signal_map_1)
            .expect("Setting signal map failed");

        let access_0 = DataAccess::new("signal1", vec![]);
        let id_0 = component
            .get_signal_id(&[0], &access_0)
            .expect("Getting signal ID failed for index 0");
        assert_eq!(id_0, 0);

        let access_1 = DataAccess::new("signal2", vec![]);
        let id_1 = component
            .get_signal_id(&[1], &access_1)
            .expect("Getting signal ID failed for index 1");
        assert_eq!(id_1, 1);
    }

    #[test]
    fn test_data_access_new() {
        let access = DataAccess::new(
            "variable",
            vec![
                SubAccess::Array(0),
                SubAccess::Component("field".to_string()),
            ],
        );
        assert_eq!(access.get_name(), "variable");
        assert_eq!(
            access.get_access(),
            &vec![
                SubAccess::Array(0),
                SubAccess::Component("field".to_string())
            ]
        );
    }

    #[test]
    fn test_data_access_set_get_name_and_access() {
        let mut access = DataAccess::new("variable", vec![]);

        // Test get_name and set_name
        assert_eq!(access.get_name(), "variable");
        access.set_name("new_variable");
        assert_eq!(access.get_name(), "new_variable");

        // Test get_access and set_access
        assert_eq!(access.get_access(), &vec![]);
        let new_access_path = vec![
            SubAccess::Array(0),
            SubAccess::Component("field".to_string()),
        ];
        access.set_access(new_access_path.clone());
        assert_eq!(access.get_access(), &new_access_path);
    }

    #[test]
    fn test_data_access_access_str() {
        let access = DataAccess::new(
            "variable",
            vec![
                SubAccess::Array(0),
                SubAccess::Component("field".to_string()),
            ],
        );
        let ctx_name = "ctx".to_string();
        let access_string = access.access_str(ctx_name.clone());
        assert_eq!(access_string, "ctx.variable[0].field");
    }

    #[test]
    fn test_data_access_access_str_with_multiple_components() {
        let access = DataAccess::new(
            "component",
            vec![
                SubAccess::Array(0),
                SubAccess::Component("subcomponent".to_string()),
                SubAccess::Array(1),
                SubAccess::Component("signal".to_string()),
            ],
        );
        let ctx_name = "ctx".to_string();
        let access_string = access.access_str(ctx_name.clone());
        assert_eq!(access_string, "ctx.component[0].subcomponent[1].signal");
    }

    #[test]
    fn test_data_access_process_component_access() {
        let access = DataAccess::new(
            "component",
            vec![
                SubAccess::Array(0),
                SubAccess::Component("signal".to_string()),
                SubAccess::Array(1),
            ],
        );

        let result = process_component_access(&access).unwrap();
        assert_eq!(
            result,
            (
                DataAccess::new("component", vec![SubAccess::Array(0)]),
                DataAccess::new("signal", vec![SubAccess::Array(1)]),
            )
        );
    }

    #[test]
    fn test_util_get_nested_value() {
        let nested_value = NestedValue::Array(vec![
            NestedValue::Value(1),
            NestedValue::Array(vec![NestedValue::Value(2), NestedValue::Value(3)]),
        ]);

        assert_eq!(
            get_nested_value(&nested_value, &[0]).unwrap(),
            NestedValue::Value(1)
        );
        assert_eq!(
            get_nested_value(&nested_value, &[1, 0]).unwrap(),
            NestedValue::Value(2)
        );
        assert_eq!(
            get_nested_value(&nested_value, &[1, 1]).unwrap(),
            NestedValue::Value(3)
        );
        assert!(get_nested_value(&nested_value, &[2]).is_err());
    }

    #[test]
    fn test_util_get_mut_nested_value() {
        let mut nested_value = NestedValue::Array(vec![
            NestedValue::Value(1),
            NestedValue::Array(vec![NestedValue::Value(2), NestedValue::Value(3)]),
        ]);

        assert_eq!(
            *get_mut_nested_value(&mut nested_value, &[0]).unwrap(),
            NestedValue::Value(1)
        );

        *get_mut_nested_value(&mut nested_value, &[1, 0]).unwrap() = NestedValue::Value(42);
        assert_eq!(
            get_nested_value(&nested_value, &[1, 0]).unwrap(),
            NestedValue::Value(42)
        );

        assert!(get_mut_nested_value(&mut nested_value, &[2]).is_err());
    }

    #[test]
    fn test_util_u32_to_access() {
        let indices = vec![0, 1, 2];
        let result = u32_to_access(&indices);
        assert_eq!(
            result,
            vec![
                SubAccess::Array(0),
                SubAccess::Array(1),
                SubAccess::Array(2),
            ]
        );
    }

    #[test]
    fn test_util_access_to_u32() {
        let sub_accesses = vec![
            SubAccess::Array(0),
            SubAccess::Array(1),
            SubAccess::Array(2),
        ];
        let result = access_to_u32(&sub_accesses).unwrap();
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_util_increment_indices() {
        let mut indices = vec![0, 0];
        let limits = vec![2, 2];
        assert!(increment_indices(&mut indices, &limits).unwrap());
        assert_eq!(indices, vec![0, 1]);

        assert!(increment_indices(&mut indices, &limits).unwrap());
        assert_eq!(indices, vec![1, 0]);

        assert!(increment_indices(&mut indices, &limits).unwrap());
        assert_eq!(indices, vec![1, 1]);

        assert!(!increment_indices(&mut indices, &limits).unwrap());
        assert_eq!(indices, vec![0, 0]);
    }
}
