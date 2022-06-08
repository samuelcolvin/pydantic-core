use pyo3::prelude::*;
use pyo3::types::{PyDate, PyType};

use crate::errors::{err_val_error, ErrorKind, InputValue, ValResult};

use super::generics::{GenericMapping, GenericSequence};
use super::input_abstract::Input;
use super::parse_json::JsonInput;
use super::shared::{date_as_py_date, float_as_int, int_as_bool, int_as_date, str_as_bool, str_as_int, string_as_date};

impl Input for JsonInput {
    fn is_none(&self) -> bool {
        matches!(self, JsonInput::Null)
    }

    fn strict_str(&self) -> ValResult<String> {
        match self {
            JsonInput::String(s) => Ok(s.to_string()),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::StrType),
        }
    }

    fn lax_str(&self) -> ValResult<String> {
        match self {
            JsonInput::String(s) => Ok(s.to_string()),
            JsonInput::Int(int) => Ok(int.to_string()),
            JsonInput::Float(float) => Ok(float.to_string()),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::StrType),
        }
    }

    fn strict_bool(&self) -> ValResult<bool> {
        match self {
            JsonInput::Bool(b) => Ok(*b),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::BoolType),
        }
    }

    fn lax_bool(&self) -> ValResult<bool> {
        match self {
            JsonInput::Bool(b) => Ok(*b),
            JsonInput::String(s) => str_as_bool(self, s),
            JsonInput::Int(int) => int_as_bool(self, *int),
            // TODO float??
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::BoolType),
        }
    }

    fn strict_int(&self) -> ValResult<i64> {
        match self {
            JsonInput::Int(i) => Ok(*i),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType),
        }
    }

    fn lax_int(&self) -> ValResult<i64> {
        match self {
            JsonInput::Bool(b) => match *b {
                true => Ok(1),
                false => Ok(0),
            },
            JsonInput::Int(i) => Ok(*i),
            JsonInput::Float(f) => float_as_int(self, *f),
            JsonInput::String(str) => str_as_int(self, str),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType),
        }
    }

    fn strict_float(&self) -> ValResult<f64> {
        match self {
            JsonInput::Float(f) => Ok(*f),
            JsonInput::Int(i) => Ok(*i as f64),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatType),
        }
    }

    fn lax_float(&self) -> ValResult<f64> {
        match self {
            JsonInput::Bool(b) => match *b {
                true => Ok(1.0),
                false => Ok(0.0),
            },
            JsonInput::Float(f) => Ok(*f),
            JsonInput::Int(i) => Ok(*i as f64),
            JsonInput::String(str) => match str.parse() {
                Ok(i) => Ok(i),
                Err(_) => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatParsing),
            },
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatType),
        }
    }

    fn strict_model_check(&self, _class: &PyType) -> ValResult<bool> {
        Ok(false)
    }

    fn strict_dict<'data>(&'data self) -> ValResult<GenericMapping<'data>> {
        match self {
            JsonInput::Object(dict) => Ok(dict.into()),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DictType),
        }
    }

    fn strict_list<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::ListType),
        }
    }

    fn strict_set<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        // we allow a list here since otherwise it would be impossible to create a set from JSON
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::SetType),
        }
    }

    fn lax_date<'data>(&'data self, py: Python<'data>) -> ValResult<&'data PyDate> {
        let date = match self {
            JsonInput::String(v) => string_as_date(self, v),
            JsonInput::Int(v) => int_as_date(self, *v),
            JsonInput::Float(v) => int_as_date(self, float_as_int(self, *v)?),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DateType),
        }?;

        return date_as_py_date!(py, date);
    }
}

/// Required for Dict keys so the string can behave like an Input
impl Input for String {
    #[no_coverage]
    fn is_none(&self) -> bool {
        false
    }

    #[no_coverage]
    fn strict_str(&self) -> ValResult<String> {
        Ok(self.clone())
    }

    #[no_coverage]
    fn lax_str(&self) -> ValResult<String> {
        Ok(self.clone())
    }

    #[no_coverage]
    fn strict_bool(&self) -> ValResult<bool> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::BoolType)
    }

    #[no_coverage]
    fn lax_bool(&self) -> ValResult<bool> {
        str_as_bool(self, self)
    }

    #[no_coverage]
    fn strict_int(&self) -> ValResult<i64> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType)
    }

    #[no_coverage]
    fn lax_int(&self) -> ValResult<i64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntParsing),
        }
    }

    #[no_coverage]
    fn strict_float(&self) -> ValResult<f64> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatType)
    }

    #[no_coverage]
    fn lax_float(&self) -> ValResult<f64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatParsing),
        }
    }

    #[no_coverage]
    fn strict_model_check(&self, _class: &PyType) -> ValResult<bool> {
        Ok(false)
    }

    #[no_coverage]
    fn strict_dict<'data>(&'data self) -> ValResult<GenericMapping<'data>> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DictType)
    }

    #[no_coverage]
    fn strict_list<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::ListType)
    }

    #[no_coverage]
    fn strict_set<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::SetType)
    }

    fn lax_date<'data>(&'data self, _py: Python<'data>) -> ValResult<&'data PyDate> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DateType)
    }
}
