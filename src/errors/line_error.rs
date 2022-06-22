use std::fmt::{Display, Formatter, Result};

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::input::JsonInput;

use super::kinds::ErrorKind;
use super::location::{owned_location, LocItem, Location};

/// A `ValLineError` is a single error that occurred during validation which is converted to a `PyLineError`
/// to eventually form a `ValidationError`.
/// I don't like the name `ValLineError`, but it's the best I could come up with (for now).
#[derive(Debug, Default)]
pub struct ValLineError<'a> {
    pub kind: ErrorKind,
    pub location: Location<'a>,
    pub message: Option<String>,
    pub input_value: InputValue<'a>,
    pub context: Context,
}

impl<'a> ValLineError<'a> {
    pub fn with_prefix_location(mut self, loc_item: LocItem<'a>) -> Self {
        self.location.insert(0, loc_item);
        self
    }

    pub fn clone_py<'b>(&'a self, py: Python) -> ValLineError<'b> {
        ValLineError {
            kind: self.kind.clone(),
            location: owned_location(&self.location),
            message: self.message.clone(),
            input_value: InputValue::PyObject(self.input_value.to_object(py)),
            context: self.context.clone(),
        }
    }
}

#[derive(Debug)]
pub enum InputValue<'a> {
    None,
    PyAny(&'a PyAny),
    JsonInput(&'a JsonInput),
    String(&'a str),
    PyObject(PyObject),
}

impl Default for InputValue<'_> {
    fn default() -> Self {
        Self::None
    }
}

impl<'a> ToPyObject for InputValue<'a> {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            Self::None => py.None(),
            Self::PyAny(input) => input.into_py(py),
            Self::JsonInput(input) => input.to_object(py),
            Self::String(input) => input.into_py(py),
            Self::PyObject(py_obj) => py_obj.into_py(py),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Context(Vec<(String, ContextValue)>);

impl Context {
    pub fn new<I: IntoIterator<Item = (String, ContextValue)>>(raw: I) -> Self {
        Self(raw.into_iter().collect())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn render(&self, template: String) -> String {
        let mut rendered = template;
        for (key, value) in &self.0 {
            rendered = rendered.replace(&format!("{{{}}}", key), &value.to_string());
        }
        rendered
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let loc = self
            .0
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{{{}}}", loc)
    }
}

// maybe this is overkill and we should just use Display an convert to string when creating Context?
#[derive(Debug, Clone)]
pub enum ContextValue {
    S(String),
    I(i64),
    F(f64),
}

impl Display for ContextValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ContextValue::S(v) => write!(f, "{}", v),
            ContextValue::I(v) => write!(f, "{}", v),
            ContextValue::F(v) => write!(f, "{}", v),
        }
    }
}

impl From<String> for ContextValue {
    fn from(str: String) -> Self {
        Self::S(str)
    }
}

impl From<&str> for ContextValue {
    fn from(str: &str) -> Self {
        Self::S(str.to_string())
    }
}

impl From<i64> for ContextValue {
    fn from(int: i64) -> Self {
        Self::I(int)
    }
}

impl From<usize> for ContextValue {
    fn from(u: usize) -> Self {
        Self::I(u as i64)
    }
}

impl From<f64> for ContextValue {
    fn from(f: f64) -> Self {
        Self::F(f)
    }
}

impl ToPyObject for ContextValue {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            ContextValue::S(v) => v.into_py(py),
            ContextValue::I(v) => v.into_py(py),
            ContextValue::F(v) => v.into_py(py),
        }
    }
}

impl ToPyObject for Context {
    fn to_object(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (key, value) in &self.0 {
            dict.set_item(key, value).unwrap();
        }
        dict.into_py(py)
    }
}
