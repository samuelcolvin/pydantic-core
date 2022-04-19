use std::str::from_utf8;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyInt, PyList, PyString};

use super::shared::{int_as_bool, str_as_bool};
use super::traits::{DictInput, Input, ListInput, ToPy};
use crate::errors::{as_internal, err_val_error, ErrorKind, ValResult};

impl ToPy for PyDict {
    fn to_py(&self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

impl ToPy for PyList {
    fn to_py(&self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

impl ToPy for PyAny {
    fn to_py(&self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

impl Input for PyAny {
    fn validate_none(&self, py: Python) -> ValResult<()> {
        if self.is_none() {
            Ok(())
        } else {
            err_val_error!(py, self, kind = ErrorKind::NoneRequired)
        }
    }

    fn validate_str(&self, py: Python) -> ValResult<String> {
        if let Ok(py_str) = self.cast_as::<PyString>() {
            py_str.extract().map_err(as_internal)
        } else if let Ok(bytes) = self.cast_as::<PyBytes>() {
            let str = match from_utf8(bytes.as_bytes()) {
                Ok(s) => s.to_string(),
                Err(_) => return err_val_error!(py, self, kind = ErrorKind::StrUnicode),
            };
            Ok(str)
        } else if let Ok(int) = self.cast_as::<PyInt>() {
            let int = i64::extract(int).map_err(as_internal)?;
            Ok(int.to_string())
        } else if let Ok(float) = f64::extract(self) {
            // don't cast_as here so Decimals are covered - internally f64:extract uses PyFloat_AsDouble
            Ok(float.to_string())
        } else {
            // let name = self.get_type().name().unwrap_or("<unknown type>");
            err_val_error!(py, self, kind = ErrorKind::StrType)
        }
    }

    fn validate_bool(&self, py: Python) -> ValResult<bool> {
        if let Ok(bool) = self.extract::<bool>() {
            Ok(bool)
        } else if let Some(str) = _maybe_as_string(py, self, ErrorKind::BoolParsing)? {
            str_as_bool(py, &str)
        } else if let Ok(int) = self.extract::<i64>() {
            int_as_bool(py, int)
        } else {
            err_val_error!(py, self, kind = ErrorKind::BoolType)
        }
    }

    fn validate_int(&self, py: Python) -> ValResult<i64> {
        if let Ok(int) = self.extract::<i64>() {
            Ok(int)
        } else if let Some(str) = _maybe_as_string(py, self, ErrorKind::IntParsing)? {
            match str.parse() {
                Ok(i) => Ok(i),
                Err(_) => err_val_error!(py, str, kind = ErrorKind::IntParsing),
            }
        } else if let Ok(float) = self.validate_float(py) {
            if float % 1.0 == 0.0 {
                Ok(float as i64)
            } else {
                err_val_error!(py, float, kind = ErrorKind::IntFromFloat)
            }
        } else {
            err_val_error!(py, self, kind = ErrorKind::IntType)
        }
    }

    fn validate_float(&self, py: Python) -> ValResult<f64> {
        if let Ok(int) = self.extract::<f64>() {
            Ok(int)
        } else if let Some(str) = _maybe_as_string(py, self, ErrorKind::FloatParsing)? {
            match str.parse() {
                Ok(i) => Ok(i),
                Err(_) => err_val_error!(py, str, kind = ErrorKind::FloatParsing),
            }
        } else {
            err_val_error!(py, self, kind = ErrorKind::FloatType)
        }
    }

    fn validate_dict<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn DictInput<'py> + 'py>> {
        if let Ok(dict) = self.cast_as::<PyDict>() {
            Ok(Box::new(PyDictInput(dict)))
            // TODO we probably want to try and support mapping like things here too
        } else {
            err_val_error!(py, self, kind = ErrorKind::DictType)
        }
    }

    fn validate_list<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn ListInput + 'py>> {
        if let Ok(list) = self.cast_as::<PyList>() {
            Ok(Box::new(PyListInput(list)))
            // TODO support sets, tuples, frozen set etc. like in pydantic
        } else {
            err_val_error!(py, self, kind = ErrorKind::ListType)
        }
    }
}

struct PyDictInput<'py>(&'py PyDict);

impl<'py> ToPy for PyDictInput<'py> {
    fn to_py(&self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

impl<'py> DictInput<'py> for PyDictInput<'py> {
    fn iter(&self) -> Box<dyn Iterator<Item = (&dyn Input, &dyn Input)> + '_> {
        Box::new(self.0.iter().map(|(k, v)| (k as &dyn Input, v as &dyn Input)))
    }

    fn get_item(&self, key: &str) -> Option<&dyn Input> {
        self.0.get_item(key).map(|item| item as &dyn Input)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

struct PyListInput<'py>(&'py PyList);

impl<'py> ToPy for PyListInput<'py> {
    fn to_py(&self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

impl<'py> ListInput<'py> for PyListInput<'py> {
    // this is ugly, is there any way to avoid the map, one of the boxes and/or avoid the duplication?
    fn iter(&self) -> Box<dyn Iterator<Item = &dyn Input> + '_> {
        Box::new(self.0.iter().map(|item| item as &dyn Input))
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Utility for extracting a string from a PyAny, if possible.
fn _maybe_as_string(py: Python, v: &PyAny, unicode_error: ErrorKind) -> ValResult<Option<String>> {
    if let Ok(str) = v.extract::<String>() {
        Ok(Some(str))
    } else if let Ok(bytes) = v.cast_as::<PyBytes>() {
        let str = match from_utf8(bytes.as_bytes()) {
            Ok(s) => s.to_string(),
            Err(_) => return err_val_error!(py, bytes, kind = unicode_error),
        };
        Ok(Some(str))
    } else {
        Ok(None)
    }
}
