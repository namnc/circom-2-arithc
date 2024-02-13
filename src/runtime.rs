//! # Runtime Module
//!
//! This module manages the main runtime, keeping track of the multiple contexts and data items in the program.

use circom_program_structure::ast::VariableType;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

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
pub struct Runtime {
    contexts: VecDeque<Context>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    /// Creates an empty runtime with no contexts.
    pub fn new() -> Self {
        Self {
            contexts: VecDeque::from([Context::new()]),
        }
    }

    /// Adds a new context onto the stack, optionally inheriting from the current context.
    pub fn push_context(&mut self, inherit: bool) -> Result<(), RuntimeError> {
        let new_context = if inherit {
            match self.contexts.front() {
                Some(parent_context) => Context::new_with_inheritance(parent_context),
                None => return Err(RuntimeError::NoContextToInheritFrom),
            }
        } else {
            Context::new()
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
}

/// Context
/// Handles a specific scope value tracking.
#[derive(Clone)]
pub struct Context {
    names: HashSet<String>,
    variables: HashMap<String, Variable>,
    signals: HashMap<String, Signal>,
    components: HashMap<String, Component>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Constructs a new Context.
    pub fn new() -> Self {
        Self {
            names: HashSet::new(),
            variables: HashMap::new(),
            signals: HashMap::new(),
            components: HashMap::new(),
        }
    }

    /// Returns a contexts that inherits from the current context.
    pub fn new_with_inheritance(&self) -> Self {
        Self {
            names: self.names.clone(),
            variables: self.variables.clone(),
            signals: self.signals.clone(),
            components: self.components.clone(),
        }
    }

    /// Merges changes from the given context into this context.
    /// Signals are not merged, as they are read-only.
    pub fn merge(&mut self, child: &Context) -> Result<(), RuntimeError> {
        for (name, variable) in &child.variables {
            if self.variables.contains_key(name) {
                self.variables.insert(name.clone(), variable.clone());
            }
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
    ) -> Result<(), RuntimeError> {
        // Check name availability
        self.add_name(name)?;
        let name_string = name.to_string();

        match data_type {
            DataType::Signal => {
                let signal = Signal::new(dimensions);
                self.signals.insert(name_string, signal);
            }
            DataType::Variable => {
                let variable = Variable::new(dimensions);
                self.variables.insert(name_string, variable);
            }
            DataType::Component => {
                let component = Component::new(dimensions);
                self.components.insert(name_string, component);
            }
        };

        Ok(())
    }

    /// Declares a new item with a random name.
    /// This might be dropped.
    pub fn declare_random_item(&mut self, data_type: DataType) -> Result<DataAccess, RuntimeError> {
        let name = format!("random_{}", generate_u32());
        self.declare_item(data_type, &name, &[])?;
        Ok(DataAccess::new(&name, vec![]))
    }

    /// Returns the data type of an item.
    pub fn get_item_data_type(&self, name: &str) -> Result<DataType, RuntimeError> {
        if self.variables.get(name).is_some() {
            Ok(DataType::Variable)
        } else if self.signals.get(name).is_some() {
            Ok(DataType::Signal)
        } else if self.components.get(name).is_some() {
            Ok(DataType::Component)
        } else {
            Err(RuntimeError::ItemNotDeclared)
        }
    }

    /// Sets the content of a variable.
    pub fn set_variable(
        &mut self,
        access: &DataAccess,
        value: Option<u32>,
    ) -> Result<(), RuntimeError> {
        let variable = self
            .variables
            .get_mut(&access.name)
            .ok_or(RuntimeError::ItemNotDeclared)?;

        variable.set(&access_to_u32(access.get_access())?, value)
    }

    /// Gets the content of a variable.
    pub fn get_variable(&self, access: &DataAccess) -> Result<Option<u32>, RuntimeError> {
        let variable = self
            .variables
            .get(&access.name)
            .ok_or(RuntimeError::ItemNotDeclared)?;

        variable.get(&access_to_u32(access.get_access())?)
    }

    /// Gets the content of a signal.
    pub fn get_signal(&self, access: &DataAccess) -> Result<u32, RuntimeError> {
        let signal = self
            .signals
            .get(&access.name)
            .ok_or(RuntimeError::ItemNotDeclared)?;

        signal.get(&access_to_u32(access.get_access())?)
    }

    /// Adds a connection in a component.
    pub fn add_connection(
        &mut self,
        component_name: &str,
        from: DataAccess,
        to: DataAccess,
    ) -> Result<(), RuntimeError> {
        let component = self
            .components
            .get_mut(component_name)
            .ok_or(RuntimeError::ItemNotDeclared)?;

        component.add_connection(from, to)
    }

    /// Checks if the name is already used and adds it to the names set.
    fn add_name(&mut self, name: &str) -> Result<(), RuntimeError> {
        if !self.names.insert(name.to_string()) {
            Err(RuntimeError::ItemAlreadyDeclared)
        } else {
            Ok(())
        }
    }
}

/// Represents a signal that holds a single id or a nested structure of values with unique IDs.
#[derive(Clone, Debug)]
struct Signal {
    value: NestedValue<u32>,
}

impl Signal {
    /// Constructs a new Signal as a nested structure based on provided dimensions.
    fn new(dimensions: &[u32]) -> Self {
        fn create_nested_signal(dimensions: &[u32]) -> NestedValue<u32> {
            if let Some((&first, rest)) = dimensions.split_first() {
                let array = (0..first).map(|_| create_nested_signal(rest)).collect();
                NestedValue::Array(array)
            } else {
                NestedValue::Value(generate_u32())
            }
        }

        Self {
            value: create_nested_signal(dimensions),
        }
    }

    /// Retrieves the ID of the signal at the specified index path.
    fn get(&self, index_path: &[u32]) -> Result<u32, RuntimeError> {
        get_nested_value(&self.value, index_path)
    }
}

/// Represents a variable that can hold a single value or nested structure of values.
#[derive(Clone, Debug)]
struct Variable {
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
        *inner_value = val;
        Ok(())
    }

    /// Retrieves the content of the variable at the specified index path.
    fn get(&self, index_path: &[u32]) -> Result<Option<u32>, RuntimeError> {
        get_nested_value(&self.value, index_path)
    }
}

/// Component
#[derive(Clone, Debug)]
pub struct Component {
    connections: NestedValue<HashMap<DataAccess, DataAccess>>,
}

impl Component {
    /// Constructs a new Component as a nested structure based on provided dimensions.
    fn new(dimensions: &[u32]) -> Self {
        let mut connections = NestedValue::Value(HashMap::new());

        // Construct the nested structure in reverse order to ensure the correct dimensionality.
        for &dimension in dimensions.iter().rev() {
            let array = vec![connections.clone(); dimension as usize];
            connections = NestedValue::Array(array);
        }

        Self { connections }
    }

    // We're not processing the `to` path since this could be another component, etc. It has to be handled later.
    /// Adds a connection from one DataAccess to another DataAccess.
    pub fn add_connection(&mut self, from: DataAccess, to: DataAccess) -> Result<(), RuntimeError> {
        if let ProcessedAccess::Component(component_path, signal_name, signal_path) =
            process_subaccess(from.get_access())?
        {
            let connections = get_mut_nested_value(&mut self.connections, &component_path)?;

            let signal_access = u32_to_access(&signal_path);
            connections.insert(DataAccess::new(&signal_name, signal_access), to);
            Ok(())
        } else {
            Err(RuntimeError::AccessError)
        }
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
}

#[derive(Debug)]
pub enum ProcessedAccess {
    Array(Vec<u32>),
    Component(Vec<u32>, String, Vec<u32>), // (initial_path, signal_name, final_path)
}

/// Processes a vector of SubAccess.
pub fn process_subaccess(sub_accesses: &[SubAccess]) -> Result<ProcessedAccess, RuntimeError> {
    let mut initial_path = Vec::new();
    let mut final_path = Vec::new();
    let mut signal_name = String::new();
    let mut has_signal = false;

    for sub_access in sub_accesses {
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
                signal_name = name.clone();
                has_signal = true;
            }
        }
    }

    if has_signal {
        Ok(ProcessedAccess::Component(
            initial_path,
            signal_name,
            final_path,
        ))
    } else {
        Ok(ProcessedAccess::Array(initial_path))
    }
}

/// Generic function to navigate through NestedValue and return the inner value.
/// The clone could be removed but then the type would need to implement Copy. Leaving it for now.
pub fn get_nested_value<T: Clone>(
    nested_value: &NestedValue<T>,
    index_path: &[u32],
) -> Result<T, RuntimeError> {
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

    match current_level {
        NestedValue::Value(inner_value) => Ok(inner_value.clone()),
        _ => Err(RuntimeError::NotAValue),
    }
}

/// Generic function to navigate through NestedValue and return a mutable reference to the inner value.
pub fn get_mut_nested_value<'a, T>(
    nested_value: &'a mut NestedValue<T>,
    index_path: &[u32],
) -> Result<&'a mut T, RuntimeError> {
    let mut current_level = nested_value;
    for &index in index_path {
        current_level = match current_level {
            NestedValue::Array(values) => values
                .get_mut(index as usize)
                .ok_or(RuntimeError::IndexOutOfBounds)?,
            _ => return Err(RuntimeError::AccessError),
        };
    }

    match current_level {
        NestedValue::Value(inner_value) => Ok(inner_value),
        _ => Err(RuntimeError::NotAValue),
    }
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
pub fn increment_indices(indices: &mut Vec<u32>, limits: &[u32]) -> Result<bool, RuntimeError> {
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

#[derive(Error, Debug, PartialEq, Eq, Clone)]
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
    #[error("Item not declared")]
    ItemNotDeclared,
    #[error("No context to inherit from")]
    NoContextToInheritFrom,
    #[error("Data Item content is not a single value")]
    NotAValue,
    #[error("Unsupported data type")]
    UnsupportedDataType,
}
