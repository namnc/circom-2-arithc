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

/// Runtime - manages the scope stack and variable tracking.
pub struct Runtime {
    ctx_stack: Vec<Context>,
    current_ctx: u32,
    last_ctx: u32,
}

impl Runtime {
    /// Constructs a new Runtime with an empty stack.
    pub fn new() -> Self {
        debug!("Creating new Runtime");
        Self {
            ctx_stack: Vec::new(),
            current_ctx: u32::default(), //TODO: why not just concretely set as 0 so we have precise control on this initial value?
            last_ctx: u32::default(),
        }
    }

    /// Creates a new context for a function call or similar operation.
    pub fn add_context(&mut self, origin: ContextOrigin) -> Result<(), RuntimeError> {
        debug!("Adding new context for origin: {:?}", origin);
        // Retrieve the caller context
        let caller_context = self.get_current_context()?;

        // Generate a unique ID for the new context
        let new_id = self.generate_context_id();

        // Create the new context using data from the caller context
        let new_context = match origin {
            ContextOrigin::Call => {
                Context::new(new_id, self.current_ctx, HashMap::new())?;
            },
            ContextOrigin::Branch => {
                Context::new(new_id, self.current_ctx, caller_context.values.clone())?;
            },
            ContextOrigin::Loop => {
                Context::new(new_id, self.current_ctx, caller_context.values.clone())?;
            },
            ContextOrigin::Block => {
                Context::new(new_id, self.current_ctx, caller_context.values.clone())?;
            },
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
        debug!("Setting content for DataItem: {:?} - {:?}", self, content);
        match self.data_type {
            DataType::Signal if self.content.is_some() => Err(RuntimeError::SignalAlreadySet),
            _ => {
                self.content = Some(content);
                Ok(())
            }
        }
    }

    /// Gets the content of the data item.
    pub fn get_content(&self) -> Option<&DataContent> {
        self.content.as_ref()
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
    pub fn get_data_type(&self) -> &DataType {
        &self.data_type
    }

    /// Checks if the content of the data item is an array.
    pub fn is_array(&self) -> bool {
        if let Some(DataContent::Array(_)) = self.content {
            true
        } else {
            false
        }
    }
}

/// Runtime errors
#[derive(Error, Debug, PartialEq, Eq)]
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
    #[error("Cannot modify an already set signal")]
    SignalAlreadySet,
}
