//! # Runtime Module
//!
//! This module manages the main runtime, keeping track of the multiple contexts and data items in the program.

use circom_program_structure::ast::VariableType;
use log::debug;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug)]
/// New context origin
pub enum ContextOrigin {
    Call,
    Branch,
    Loop,
    Block,
}

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

/// Runtime - manages the context stack and variable tracking.
pub struct Runtime {
    ctx_stack: Vec<Context>,
    current_ctx: u32,
    _last_ctx: u32,
}

impl Runtime {
    /// Constructs a new Runtime with an empty stack.
    pub fn new() -> Result<Self, RuntimeError> {
        debug!("New runtime");
        Ok(Self {
            ctx_stack: vec![Context::new(0, 0)],
            current_ctx: 0,
            _last_ctx: 0,
        })
    }

    /// Creates a new context for a function call or similar operation.
    pub fn add_context(&mut self, origin: ContextOrigin) -> Result<(), RuntimeError> {
        debug!("New context - origin: {:?}", origin);
        // Generate a unique ID for the new context
        let new_id = self.generate_context_id();
        let current_context = self.get_current_context()?;

        // Create the new context using data from the caller context
        let new_context = match origin {
            ContextOrigin::Call => Context::new(new_id, self.current_ctx),
            ContextOrigin::Branch => current_context.new_child(new_id),
            ContextOrigin::Loop => current_context.new_child(new_id),
            ContextOrigin::Block => current_context.new_child(new_id),
        };

        // NOTE: above could be the simplest way to do what I wrote below. We just let Call be with an empty map and there will be declaration and initialization coming in from operations code.
        // TODO: we might want to distiguish the context creation reason here
        // this behavior right now is good for if_then_else, loop, and block because all variables and signals declare in the caller are accessible inside those
        // for template and function call we don't really have access to variables and signals outside of the template and function definition
        // for template we pass some values to initialize the variables define in the args of the template and inside the function we can actually re-declare variables and signals.
        // for function we also pass some values to initialize the variables define in the args of the function and inside the function we can actually re-declare variables (no signals in functions).

        // Push the new context onto the stack and update current_ctx
        self.ctx_stack.push(new_context);
        self.current_ctx = new_id;

        debug!(
            "New context added successfully. Current context ID: {}",
            self.current_ctx
        );

        Ok(())
    }

    /// Retrieves a specific context by its ID.
    pub fn get_context(&mut self, id: u32) -> Result<&mut Context, RuntimeError> {
        self.ctx_stack
            .iter_mut()
            .find(|ctx| ctx.id == id)
            .ok_or(RuntimeError::ContextNotFound)
    }

    /// Pop context and return to the caller
    pub fn pop_context(&mut self) -> Result<&mut Context, RuntimeError> {
        let id = self.current_ctx;
        self.ctx_stack
            .iter_mut()
            .find(|ctx| ctx.id == id)
            .ok_or(RuntimeError::ContextNotFound)
    }

    /// Retrieves the current runtime context.
    pub fn get_current_context(&mut self) -> Result<&mut Context, RuntimeError> {
        self.get_context(self.current_ctx)
    }

    /// Generates a unique context ID.
    fn generate_context_id(&mut self) -> u32 {
        thread_rng().gen()
    }
}

/// Context
/// Handles a specific scope value tracking.
#[derive(Clone)]
pub struct Context {
    id: u32,
    caller_id: u32,
    names: HashSet<String>,
    variables: HashMap<String, Variable>,
    signals: HashMap<String, Signal>,
    components: HashMap<String, Component>,
}

impl Context {
    /// Constructs a new Context.
    pub fn new(id: u32, caller_id: u32) -> Self {
        Self {
            id,
            caller_id,
            names: HashSet::new(),
            variables: HashMap::new(),
            signals: HashMap::new(),
            components: HashMap::new(),
        }
    }

    /// Returns a contexts that inherits from the current context.
    pub fn new_child(&self, id: u32) -> Self {
        Self {
            id,
            caller_id: self.id,
            names: self.names.clone(),
            variables: self.variables.clone(),
            signals: self.signals.clone(),
            components: self.components.clone(),
        }
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
        let name = format!("random_{}", self.generate_id());
        self.declare_item(data_type, &name, &[])?;
        Ok(DataAccess::new(name, vec![]))
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
        access: DataAccess,
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

    /// Returns the caller context id.
    pub fn caller_id(&self) -> u32 {
        self.caller_id
    }

    /// Checks if the name is already used and adds it to the names set.
    fn add_name(&mut self, name: &str) -> Result<(), RuntimeError> {
        if !self.names.insert(name.to_string()) {
            Err(RuntimeError::ItemAlreadyDeclared)
        } else {
            Ok(())
        }
    }

    /// Generates a random u32 ID.
    fn generate_id(&self) -> u32 {
        thread_rng().gen()
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
        let mut rng = rand::thread_rng();

        // Create nested signals with unique IDs.
        fn create_nested_signal(dimensions: &[u32], rng: &mut impl Rng) -> NestedValue<u32> {
            if let Some((&first, rest)) = dimensions.split_first() {
                let array = (0..first)
                    .map(|_| create_nested_signal(rest, rng))
                    .collect();
                NestedValue::Array(array)
            } else {
                NestedValue::Value(rng.gen())
            }
        }

        Self {
            value: create_nested_signal(dimensions, &mut rng),
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
            connections.insert(DataAccess::new(signal_name, signal_access), to);
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
    pub fn new(name: String, access: Vec<SubAccess>) -> Self {
        Self { name, access }
    }

    /// Gets the name of the data item.
    pub fn get_name(&self) -> String {
        self.name.clone()
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

/// Runtime errors
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum RuntimeError {
    #[error("Access Error")]
    AccessError,
    #[error("Error retrieving context")]
    ContextRetrievalError,
    #[error("Context not found")]
    ContextNotFound,
    #[error("Empty context stack")]
    EmptyContextStack,
    #[error("Index out of bounds")]
    IndexOutOfBounds,
    #[error("Item already declared")]
    ItemAlreadyDeclared,
    #[error("Item not declared")]
    ItemNotDeclared,
    #[error("Data Item content is not an array")]
    NotAnArray,
    #[error("Data Item content is not a single value")]
    NotAValue,
    #[error("Unsuported data type")]
    UnsupportedDataType,
}
