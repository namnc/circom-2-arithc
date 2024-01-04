//! # Runtime Module
//!
//! This module manages the main runtime, keeping track of the multiple contexts and data items in the program.

use log::debug;
use std::collections::HashMap;
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
        debug!("Creating new Runtime");
        Ok(Self {
            ctx_stack: vec![Context::new(0, 0, HashMap::new())?],
            current_ctx: 0,
            last_ctx: 0,
        })
    }

    /// Creates a new context for a function call or similar operation.
    pub fn add_context(&mut self, origin: ContextOrigin) -> Result<(), RuntimeError> {
        debug!("Adding new context for origin: {:?}", origin);
        // Generate a unique ID for the new context
        let new_id = self.generate_context_id();

        // Create the new context using data from the caller context
        let values = match origin {
            ContextOrigin::Call => HashMap::new(),
            ContextOrigin::Branch => self.get_current_context()?.values.clone(),
            ContextOrigin::Loop => self.get_current_context()?.values.clone(),
            ContextOrigin::Block => self.get_current_context()?.values.clone(),
        };
        let new_context = Context::new(new_id, self.current_ctx, values)?;

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
    pub fn new(
        id: u32,
        caller_id: u32,
        values: HashMap<String, DataItem>,
    ) -> Result<Self, RuntimeError> {
        debug!("Creating new context with id: {}", id);
        Ok(Self {
            id,
            caller_id,
            values,
        })
    }

    /// Declares a new data item in the context with the given name and data type.
    /// Returns an error if the data item is already declared.
    pub fn declare_data_item(
        &mut self,
        name: &str,
        data_type: DataType,
    ) -> Result<(), RuntimeError> {
        debug!("Declaring data item '{}' with type {:?}", name, data_type);
        if self.values.contains_key(name) {
            Err(RuntimeError::DataItemAlreadyDeclared)
        } else {
            self.values
                .insert(name.to_string(), DataItem::new(data_type));
            Ok(())
        }
    }

    /// Assigns a value to a data item in the context.
    /// Returns an error if the data item is not found.
    pub fn set_data_item(&mut self, name: &str, content: DataContent) -> Result<(), RuntimeError> {
        debug!("Setting content for data item '{}'", name);
        match self.values.get_mut(name) {
            Some(data_item) => data_item.set_content(content),
            None => Err(RuntimeError::DataItemNotDeclared),
        }
    }

    /// Retrieves a reference to a data item by name.
    /// Returns an error if the data item is not found.
    pub fn get_data_item(&self, name: &str) -> Result<&DataItem, RuntimeError> {
        self.values
            .get(name)
            .ok_or(RuntimeError::DataItemNotDeclared)
    }

    /// Removes a data item from the context.
    /// Returns an error if the data item is not found.
    pub fn remove_data_item(&mut self, name: &str) -> Result<(), RuntimeError> {
        debug!("Removing data item '{}'", name);
        if self.values.remove(name).is_some() {
            Ok(())
        } else {
            Err(RuntimeError::DataItemNotDeclared)
        }
    }

    /// Clears the content of a data item in the context.
    /// Returns an error if the data item is not found.
    pub fn clear_data_item(&mut self, name: &str) -> Result<(), RuntimeError> {
        debug!("Clearing content for data item '{}'", name);
        match self.values.get_mut(name) {
            Some(data_item) => {
                data_item.clear_content();
                Ok(())
            }
            None => Err(RuntimeError::DataItemNotDeclared),
        }
    }

    /// Declares a new variable.
    pub fn declare_variable(&mut self, name: &str) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) {
            Err(RuntimeError::DataItemAlreadyDeclared)
        } else {
            debug!("Declaring variable '{}'", name);
            self.values
                .insert(name.to_string(), DataItem::new(DataType::Variable));
            Ok(())
        }
    }

    /// Declares a new signal.
    pub fn declare_signal(&mut self, name: &str) -> Result<u32, RuntimeError> {
        let signal_id = self.generate_id();
        if self.values.contains_key(name) {
            Err(RuntimeError::DataItemAlreadyDeclared)
        } else {
            debug!("Declaring signal '{}'", name);
            self.values
                .insert(name.to_string(), DataItem::new(DataType::Signal));
            self.set_data_item(name, DataContent::Scalar(signal_id))?;
            Ok(signal_id)
        }
    }

    /// Declares a new const value as a signal.
    /// Sets the value of the signal to the given value. This being the signal id.
    pub fn declare_const(&mut self, value: u32) -> Result<(), RuntimeError> {
        let const_name = value.to_string();
        if self.values.contains_key(&const_name) {
            Err(RuntimeError::DataItemAlreadyDeclared)
        } else {
            debug!("Declaring const '{}'", const_name);
            self.values
                .insert(const_name.clone(), DataItem::new(DataType::Signal));
            self.set_data_item(&const_name, DataContent::Scalar(value))?;
            Ok(())
        }
    }

    /// Declares a new auto generated variable.
    pub fn declare_auto_var(&mut self) -> Result<String, RuntimeError> {
        let auto_var_name = format!("auto_var_{}", self.generate_id());
        if self.values.contains_key(&auto_var_name) {
            Err(RuntimeError::DataItemAlreadyDeclared)
        } else {
            debug!("Declaring auto generated variable '{}'", auto_var_name);
            self.values
                .insert(auto_var_name.clone(), DataItem::new(DataType::Variable));
            Ok(auto_var_name)
        }
    }

    /// Gets the value of a const signal.
    pub fn get_const(&self, value: u32) -> Result<&DataItem, RuntimeError> {
        let const_name = value.to_string();
        self.get_data_item(&const_name)
    }

    // DataItem id generation needs to be reviewed. There could be collisions between
    // different contexts due to constant signals haivng arbitrary values.
    /// Generates a unique ID for a DataItem based on the context. (Temporary implementation)
    pub fn generate_id(&mut self) -> u32 {
        self.values.len() as u32 * 2 + 1000
    }

    // TODO: array auto var should support multi-dimension, right now 1
    // TODO: temporary implementation, need to be reviewed
    /// Creates a unique signal for an array element based on its indices and assigns it a unique identifier.
    pub fn declare_signal_array(
        &mut self,
        name: &str,
        indices: Vec<u32>,
    ) -> Result<(String, u32), RuntimeError> {
        let mut signal_name = name.to_string();

        for indice in indices {
            signal_name.push_str(&format!("_{}", indice));
        }

        let signal_id = self.declare_signal(&signal_name)?;

        Ok((signal_name, signal_id))
    }

    /// Creates a unique variable for an array element based on its indices and assigns it a unique identifier.
    pub fn declare_var_array(&mut self, name: &str, indices: Vec<u32>) -> Result<(), RuntimeError> {
        let mut var_name = name.to_string();

        for indice in indices {
            var_name.push_str(&format!("_{}", indice));
        }

        self.declare_variable(name)
    }
}

/// Data type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataType {
    Signal,
    Variable,
}

/// Data content
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataContent {
    Scalar(u32),
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
        debug!("Setting content {:?} - {:?}", self.data_type, content);
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
    #[error("Cannot modify an already set signal")]
    SignalAlreadySet,
}
