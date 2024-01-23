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
    Signal,
    Variable,
    Component,
}

impl TryFrom<&VariableType> for DataType {
    type Error = RuntimeError;
    fn try_from(t: &VariableType) -> Result<Self, Self::Error> {
        match t {
            VariableType::Signal(..) => Ok(DataType::Signal),
            VariableType::Var => Ok(DataType::Variable),
            VariableType::Component => Ok(DataType::Component),
            _ => Err(RuntimeError::UnsupportedVariableType),
        }
    }
}

/// Data content
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataContent {
    Scalar(u32),
    Array(Vec<Option<DataContent>>),
}

/// Runtime - manages the context stack and variable tracking.
pub struct Runtime {
    ctx_stack: Vec<Context>,
    current_ctx: u32,
    last_ctx: u32,
}

impl Runtime {
    /// Constructs a new Runtime with an empty stack.
    pub fn new() -> Result<Self, RuntimeError> {
        debug!("New runtime");
        Ok(Self {
            ctx_stack: vec![Context::new(0, 0)],
            current_ctx: 0,
            last_ctx: 0,
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
            names: self.names,
            variables: self.variables,
            signals: self.signals,
            components: self.components,
        }
    }

    /// Declares a new item of the specified type with the given name.
    pub fn declare_item(&mut self, name: &str, data_type: DataType) -> Result<(), RuntimeError> {
        match data_type {
            DataType::Variable => self.declare_variable(name),
            DataType::Signal => self.declare_signal(name),
            DataType::Component => self.declare_component(name),
        }
    }

    /// Declares a new item with a random name.
    pub fn declare_random_item(&mut self, data_type: DataType) -> Result<String, RuntimeError> {
        let name = format!("item_{}", self.generate_id());
        self.declare_item(&name, data_type)?;
        Ok(name)
    }

    /// Returns the data type of an item.
    pub fn get_item_data_type(&self, name: &str) -> Result<DataType, RuntimeError> {
        let variable = self.variables.get(name);
        let signal = self.signals.get(name);
        let component = self.components.get(name);

        if let Some(variable) = variable {
            Ok(variable.get_data_type())
        } else if let Some(signal) = signal {
            Ok(signal.get_data_type())
        } else if let Some(component) = component {
            Ok(component.get_data_type())
        } else {
            Err(RuntimeError::ItemNotDeclared)
        }
    }

    /// Declares a new variable.
    pub fn declare_variable(&mut self, name: &str) -> Result<(), RuntimeError> {
        self.add_name(name)?;
        self.variables
            .insert(name.to_string(), Variable::new(None, false));
        Ok(())
    }

    /// Sets the content of a variable.
    pub fn set_variable(&mut self, name: &str, content: DataContent) -> Result<(), RuntimeError> {
        let variable = self
            .variables
            .get_mut(name)
            .ok_or(RuntimeError::ItemNotDeclared)?;
        variable.set(content)?;
        Ok(())
    }

    /// Gets the content of a variable.
    pub fn get_variable(&self, data_access: &DataAccess) -> Result<Option<u32>, RuntimeError> {
        let variable = self
            .variables
            .get(&data_access.name)
            .ok_or(RuntimeError::ItemNotDeclared)?;
        variable.get(data_access.sub_access.clone())
    }

    /// Declares a new signal.
    pub fn declare_signal(&mut self, name: &str) -> Result<(), RuntimeError> {
        self.add_name(name)?;
        let signal_id = self.generate_id();
        self.signals.insert(
            name.to_string(),
            Signal::new(DataContent::Scalar(signal_id)),
        );
        Ok(())
    }

    /// Gets the content of a signal.
    pub fn get_signal(&self, data_access: &DataAccess) -> Result<u32, RuntimeError> {
        let signal = self
            .signals
            .get(&data_access.name)
            .ok_or(RuntimeError::ItemNotDeclared)?;
        signal.get(data_access.sub_access.clone())
    }

    /// Declares a new component.
    pub fn declare_component(&mut self, name: &str) -> Result<(), RuntimeError> {
        self.add_name(name)?;
        self.components.insert(name.to_string(), Component::new());
        Ok(())
    }

    /// Gets a component.
    pub fn get_component(&self, name: &str) -> Result<&Component, RuntimeError> {
        self.components
            .get(name)
            .ok_or(RuntimeError::ItemNotDeclared)
    }

    /// Declares an array of variables or signals.
    pub fn declare_array(
        &mut self,
        name: &str,
        data_type: DataType,
        dimensions: Vec<u32>,
    ) -> Result<(), RuntimeError> {
        self.add_name(name)?;

        if dimensions.is_empty() {
            return Err(RuntimeError::NotAnArray);
        }

        let mut array: Vec<DataContent> = Vec::new();

        for dimension in dimensions {
            if dimension == 0 {
                return Err(RuntimeError::IndexOutOfBounds);
            }
        }

        // TODO
        Ok(())
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

/// Signal
#[derive(Clone, Debug)]
struct Signal {
    id: DataContent,
}

impl Signal {
    /// Constructs a new Signal.
    pub fn new(id: DataContent) -> Self {
        Self { id }
    }

    /// Gets the id of the specified signal.
    pub fn get(&self, access: Option<SubAccess>) -> Result<u32, RuntimeError> {
        match access {
            None => match self.id {
                DataContent::Scalar(value) => Ok(value),
                _ => Err(RuntimeError::AccessError), // Access is None but the content is an array.
            },
            Some(_) => match access_content(access, self.id.clone()) {
                Ok(Some(value)) => Ok(value),
                Ok(None) => Err(RuntimeError::EmptyDataItem), // This shoulnd't happen. Signals should always have an id.
                Err(e) => Err(e),
            },
        }
    }

    /// Gets the data type of the data item.
    pub fn get_data_type(&self) -> DataType {
        DataType::Signal
    }

    /// Checks if the id is an array.
    pub fn is_array(&self) -> bool {
        matches!(self.id, DataContent::Array(_))
    }
}

/// Variable
#[derive(Clone, Debug)]
struct Variable {
    value: Option<DataContent>,
    is_constant: bool,
}

impl Variable {
    /// Constructs a new Signal.
    pub fn new(value: Option<DataContent>, is_constant: bool) -> Self {
        Self { value, is_constant }
    }

    /// Sets the variable content.
    pub fn set(&self, value: DataContent) -> Result<(), RuntimeError> {
        if self.is_constant {
            return Err(RuntimeError::VariableAlreadySet);
        }

        self.value = Some(value);

        Ok(())
    }

    /// Gets the content of the data item.
    pub fn get(&self, access: Option<SubAccess>) -> Result<Option<u32>, RuntimeError> {
        match (&access, &self.value) {
            (None, Some(DataContent::Scalar(value))) => Ok(Some(*value)),
            (Some(_), Some(content)) => access_content(access, content.clone()),
            (Some(_), None) => Err(RuntimeError::AccessError),
            (None, _) => Ok(None),
        }
    }

    /// Gets the data type of the data item.
    pub fn get_data_type(&self) -> DataType {
        DataType::Variable
    }

    /// Checks if the id is an array.
    pub fn is_array(&self) -> bool {
        matches!(self.value, Some(DataContent::Array(_)))
    }
}

/// Component
#[derive(Clone, Debug)]
struct Component {
    wiring: HashMap<DataAccess, DataAccess>,
}

impl Component {
    /// Constructs a new Signal.
    pub fn new() -> Self {
        Self {
            wiring: HashMap::new(),
        }
    }

    /// Adds a connection to the component wiring.
    pub fn add_connection(&self, from: DataAccess, to: DataAccess) -> Result<(), RuntimeError> {
        self.wiring.insert(from, to);
        Ok(())
    }

    /// Gets the wiring map.
    pub fn get_wiring(&self) -> HashMap<DataAccess, DataAccess> {
        self.wiring.clone()
    }

    /// Gets the data type of the data item.
    pub fn get_data_type(&self) -> DataType {
        DataType::Component
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataAccess {
    name: String,
    sub_access: Option<SubAccess>,
}

impl DataAccess {
    /// Constructs a new DataAccess.
    pub fn new(name: String, sub_access: Option<SubAccess>) -> Self {
        Self { name, sub_access }
    }

    /// Gets the name of the data item.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Gets the sub access of the data item.
    pub fn get_sub_access(&self) -> Option<SubAccess> {
        self.sub_access.clone()
    }
}

/// Sub data access
/// If the item is a component, the component property will be Some, and it will contain the name of the signal.
/// If the item is also an array, the array vec contains the length of the array in each dimension.
/// e.g. [2,2] for a 2x2 array.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SubAccess {
    component: Option<String>,
    array: Vec<u32>,
}

impl SubAccess {
    /// Constructs a new DataAccess.
    pub fn new(component: Option<String>, array: Vec<u32>) -> Self {
        Self { component, array }
    }
}

/// Gets the inner content of a given item data content with the specified access.
pub fn access_content(
    access: Option<SubAccess>,
    content: DataContent,
) -> Result<Option<u32>, RuntimeError> {
    match access {
        Some(sub_access) => {
            let mut current_content = Some(content);
            for (i, index) in sub_access.array.iter().enumerate() {
                match current_content {
                    Some(DataContent::Array(array_content)) => {
                        // Check if the index is within the bounds of the current array content
                        if let Some(inner_content_option) = array_content.get(*index as usize) {
                            // If we've processed all indices, check if the current content is a scalar
                            if i == sub_access.array.len() - 1 {
                                match inner_content_option {
                                    Some(DataContent::Scalar(value)) => return Ok(Some(*value)),
                                    None => return Ok(None),
                                    _ => return Err(RuntimeError::NotAScalar), // Item is not a scalar
                                }
                            }

                            // Update current_content to continue navigation
                            current_content = inner_content_option.clone();
                        } else {
                            return Err(RuntimeError::IndexOutOfBounds);
                        }
                    }

                    // Invalid access: Non-array item before processing all indices
                    Some(_) => return Err(RuntimeError::AccessError),
                    None => return Ok(None),
                }
            }

            Err(RuntimeError::NotAScalar)
        }
        None => match content {
            DataContent::Scalar(value) => Ok(Some(value)),
            DataContent::Array(_) => Err(RuntimeError::NotAScalar),
        },
    }
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
    #[error("Empty data item")]
    EmptyDataItem,
    #[error("Index out of bounds")]
    IndexOutOfBounds,
    #[error("Item already declared")]
    ItemAlreadyDeclared,
    #[error("Item not declared")]
    ItemNotDeclared,
    #[error("Data Item content is not an array")]
    NotAnArray,
    #[error("Data Item content is not a scalar")]
    NotAScalar,
    #[error("Data Item content is not a component wiring")]
    NotAWiring,
    #[error("Unsuported variable type")]
    UnsupportedVariableType,
    #[error("Constant variable already set")]
    VariableAlreadySet,
}
