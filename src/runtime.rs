//! # Runtime Module
//!
//! This module manages the main runtime, keeping track of the multiple contexts and data items in the program.

use circom_program_structure::ast::VariableType;
use log::debug;
use rand::{thread_rng, Rng};
use std::collections::{hash_map::Entry, HashMap};
use thiserror::Error;

#[derive(Debug)]
/// New context origin
pub enum ContextOrigin {
    Call,
    Branch,
    Loop,
    Block,
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
            ctx_stack: vec![Context::new(0, 0, HashMap::new())],
            current_ctx: 0,
            last_ctx: 0,
        })
    }

    /// Creates a new context for a function call or similar operation.
    pub fn add_context(&mut self, origin: ContextOrigin) -> Result<(), RuntimeError> {
        debug!("New context - origin: {:?}", origin);
        // Generate a unique ID for the new context
        let new_id = self.generate_context_id();

        // Create the new context using data from the caller context
        let values = match origin {
            ContextOrigin::Call => HashMap::new(),
            ContextOrigin::Branch => self.get_current_context()?.values.clone(),
            ContextOrigin::Loop => self.get_current_context()?.values.clone(),
            ContextOrigin::Block => self.get_current_context()?.values.clone(),
        };
        let new_context = Context::new(new_id, self.current_ctx, values);

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
        self.last_ctx += 1;
        self.last_ctx
    }
}

/// Context
/// Handles a specific scope value tracking.
#[derive(Clone)]
pub struct Context {
    id: u32,
    #[allow(dead_code)]
    caller_id: u32,
    values: HashMap<String, DataItem>, // Name -> Value
}

impl Context {
    /// Constructs a new Context.
    pub fn new(id: u32, caller_id: u32, values: HashMap<String, DataItem>) -> Self {
        debug!("New context - id: {}", id);
        Self {
            id,
            caller_id,
            values,
        }
    }

    /// Declares a new data item.
    pub fn declare_data_item(
        &mut self,
        name: &str,
        data_type: DataType,
    ) -> Result<(), RuntimeError> {
        self.declare_item(name, DataItem::new(data_type))
    }

    /// Sets the content of a data item.    
    pub fn set_data_item(&mut self, name: &str, content: DataContent) -> Result<(), RuntimeError> {
        debug!("Setting data item {} - {:?}", name, content);
        self.values
            .get_mut(name)
            .ok_or(RuntimeError::DataItemNotDeclared)?
            .set_content(content)
    }

    /// Clears the content of a data item.
    pub fn clear_data_item(&mut self, name: &str) -> Result<(), RuntimeError> {
        debug!("Clearing data item {}", name);
        self.values
            .get_mut(name)
            .ok_or(RuntimeError::DataItemNotDeclared)?
            .clear_content();
        Ok(())
    }

    /// Gets the content of a data item.
    pub fn get_data_item(&self, name: &str) -> Result<&DataItem, RuntimeError> {
        debug!("Getting data item {}", name);
        self.values
            .get(name)
            .ok_or(RuntimeError::DataItemNotDeclared)
    }

    /// Removes a data item from the context.
    pub fn remove_data_item(&mut self, name: &str) -> Result<(), RuntimeError> {
        debug!("Removing data item {}", name);
        self.values
            .remove(name)
            .map(|_| ())
            .ok_or(RuntimeError::DataItemNotDeclared)
    }

    /// Declares a new variable.
    pub fn declare_variable(&mut self, name: &str) -> Result<(), RuntimeError> {
        self.declare_item(name, DataItem::new(DataType::Variable))
    }

    /// Declares a new signal.
    pub fn declare_signal(&mut self, name: &str) -> Result<u32, RuntimeError> {
        debug!("Declaring signal {}", name);
        let signal_id = self.generate_id();
        self.declare_item(name, DataItem::new(DataType::Signal))?;
        self.set_data_item(name, DataContent::Scalar(signal_id))?;
        Ok(signal_id)
    }

    /// Declares a new component.
    pub fn declare_component(&mut self, name: &str) -> Result<u32, RuntimeError> {
        debug!("Declaring component {}", name);
        let component_id = self.generate_id();
        self.declare_item(name, DataItem::new(DataType::Component))?;
        self.set_data_item(name, DataContent::Wiring(HashMap::new()))?;
        Ok(component_id)
    }

    /// Declares a new constant.
    pub fn declare_const(&mut self, value: u32) -> Result<(), RuntimeError> {
        debug!("Declaring const {:?}", value);
        let const_name = value.to_string();
        self.declare_item(&const_name, DataItem::new(DataType::Signal))?;
        self.set_data_item(&const_name, DataContent::Scalar(value))
    }

    /// Declares an auto generated variable.
    pub fn declare_auto_var(&mut self) -> Result<String, RuntimeError> {
        let auto_name = format!("auto_var_{}", self.generate_id());
        debug!("Declaring auto generated variable {}", auto_name);
        self.declare_item(&auto_name, DataItem::new(DataType::Variable))?;
        Ok(auto_name)
    }

    /// Declares an auto generated signal.
    pub fn declare_auto_signal(&mut self) -> Result<String, RuntimeError> {
        let signal_id = self.generate_id();
        let auto_name = format!("auto_signal_{}", signal_id);
        debug!("Declaring auto generated signal {}", auto_name);
        self.declare_item(&auto_name, DataItem::new(DataType::Signal))?;
        self.set_data_item(&auto_name, DataContent::Scalar(signal_id))?;
        Ok(auto_name)
    }

    /// Declares an array of signals, variables or components.
    pub fn declare_array(
        &mut self,
        name: &str,
        data_type: DataType,
        dimensions: Vec<u32>,
    ) -> Result<(), RuntimeError> {
        debug!("Declaring array: {} - {:?}", name, data_type);
        self.declare_item(name, DataItem::new(data_type.clone()))?;

        if dimensions.is_empty() {
            return Err(RuntimeError::NotAnArray);
        }

        let mut array: Vec<DataContent> = Vec::new();

        for dimension in dimensions {
            if dimension == 0 {
                return Err(RuntimeError::IndexOutOfBounds);
            }
        }

        self.set_data_item(name, DataContent::Array(array))
    }

    /// Gets the value of a constant.
    pub fn get_const(&self, value: u32) -> Result<&DataItem, RuntimeError> {
        let const_name = value.to_string();
        self.get_data_item(&const_name)
    }

    /// Generates a random u32 ID.
    fn generate_id(&self) -> u32 {
        thread_rng().gen()
    }

    /// Declares a new item.
    pub fn declare_item(&mut self, name: &str, data_item: DataItem) -> Result<(), RuntimeError> {
        match self.values.entry(name.to_string()) {
            Entry::Occupied(_) => Err(RuntimeError::DataItemAlreadyDeclared),
            Entry::Vacant(entry) => {
                entry.insert(data_item);
                Ok(())
            }
        }
    }
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
    Wiring(HashMap<String, String>),
    Array(Vec<DataContent>),
}

/// Data item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataItem {
    data_type: DataType,
    content: Option<DataContent>,
}

impl DataItem {
    /// Constructs a new DataItem.
    pub fn new(data_type: DataType) -> Self {
        Self {
            data_type,
            content: None,
        }
    }

    /// Sets the content of the data item. Returns an error if the item is a signal and is already set.
    pub fn set_content(&mut self, content: DataContent) -> Result<(), RuntimeError> {
        match self.data_type {
            DataType::Signal if self.content.is_some() => Err(RuntimeError::SignalAlreadySet),
            _ => {
                self.content = Some(content);
                Ok(())
            }
        }
    }

    /// Clears the content of the data item.
    pub fn clear_content(&mut self) {
        self.content = None;
    }

    /// Gets the content of the data item.
    pub fn get_content(&self) -> Option<&DataContent> {
        self.content.as_ref()
    }

    /// Gets the u32 value if the content is a scalar.
    /// Returns an error if the content is an array or not set.
    pub fn get_u32(&self) -> Result<u32, RuntimeError> {
        match &self.content {
            Some(DataContent::Scalar(value)) => Ok(*value),
            Some(DataContent::Array(_)) => Err(RuntimeError::NotAScalar),
            Some(DataContent::Wiring(_)) => Err(RuntimeError::NotAScalar),
            None => Err(RuntimeError::EmptyDataItem),
        }
    }

    /// Gets the name value if the content is a wiring.
    /// Returns an error if the content is not a wiring or not set.
    pub fn get_wiring(&self) -> Result<HashMap<String, String>, RuntimeError> {
        match &self.content {
            Some(DataContent::Scalar(_)) => Err(RuntimeError::NotAWiring),
            Some(DataContent::Array(_)) => Err(RuntimeError::NotAWiring),
            Some(DataContent::Wiring(value)) => Ok(value.clone()),
            None => Err(RuntimeError::EmptyDataItem),
        }
    }

    /// Retrieves an item from the array content at the specified index.
    /// Returns an error if the content is not an array or the index is out of bounds.
    pub fn get_array_item(&self, index: usize) -> Result<&DataContent, RuntimeError> {
        match &self.content {
            Some(DataContent::Array(array)) => {
                array.get(index).ok_or(RuntimeError::IndexOutOfBounds)
            }
            Some(DataContent::Scalar(_)) => Err(RuntimeError::NotAnArray),
            Some(DataContent::Wiring(_)) => Err(RuntimeError::NotAnArray),
            None => Err(RuntimeError::EmptyDataItem),
        }
    }

    /// Gets the data type of the data item.
    pub fn get_data_type(&self) -> DataType {
        self.data_type.clone()
    }

    /// Checks if the content of the data item is an array.
    pub fn is_array(&self) -> bool {
        matches!(self.content, Some(DataContent::Array(_)))
    }
}

/// Runtime errors
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum RuntimeError {
    #[error("Error retrieving context")]
    ContextRetrievalError,
    #[error("Context not found")]
    ContextNotFound,
    #[error("Data Item already declared")]
    DataItemAlreadyDeclared,
    #[error("Data Item not declared")]
    DataItemNotDeclared,
    #[error("Empty context stack")]
    EmptyContextStack,
    #[error("Empty data item")]
    EmptyDataItem,
    #[error("Index out of bounds")]
    IndexOutOfBounds,
    #[error("Data Item content is not an array")]
    NotAnArray,
    #[error("Data Item content is not a scalar")]
    NotAScalar,
    #[error("Data Item content is not a component wiring")]
    NotAWiring,
    #[error("Cannot modify an already set signal")]
    SignalAlreadySet,
    #[error("Unsuported variable type")]
    UnsupportedVariableType,
}
