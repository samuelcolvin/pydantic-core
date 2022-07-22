use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{ErrorKind, ValError, ValLineError, ValResult};
use crate::input::{GenericListLike, GenericMapping, Input};
use crate::recursion_guard::RecursionGuard;

use super::tuple::{TuplePositionalValidator, TupleVariableValidator};
use super::typed_dict::TypedDictValidator;
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ArgumentsValidator {
    positional_only_args: Option<TuplePositionalValidator>,
    var_args: Option<TupleVariableValidator>,
    keyword_args: Option<TypedDictValidator>,
    function: Option<PyObject>,
    name: String,
}

impl BuildValidator for ArgumentsValidator {
    const EXPECTED_TYPE: &'static str = "arguments";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();

        macro_rules! build_specific_validator {
            ($key:literal, $enum_key:ident) => {
                match schema.get_item(intern!(py, $key)) {
                    Some(sub_schema) => match build_validator(sub_schema, config, build_context)?.0 {
                        CombinedValidator::$enum_key(v) => Some(v),
                        _ => return py_error!("Wrong validator type from {}", $key),
                    },
                    None => None,
                }
            };
        }

        let function: Option<&PyAny> = schema.get_item(intern!(py, "function"));
        let name = match function {
            Some(f) => {
                let function_name: &str = f.getattr(intern!(py, "__name__"))?.extract()?;
                format!("{}[{}]", Self::EXPECTED_TYPE, function_name)
            }
            None => Self::EXPECTED_TYPE.to_string(),
        };
        let s = Self {
            positional_only_args: build_specific_validator!("positional_only_args", TuplePositional),
            var_args: build_specific_validator!("var_args", TupleVariable),
            keyword_args: build_specific_validator!("keyword_args", TypedDict),
            function: function.map(|f| f.into_py(py)),
            name,
        };
        if s.positional_only_args.is_none() && s.var_args.is_none() && s.keyword_args.is_none() {
            py_error!(
                "arguments validator must have at least one of 'positional_only_args', 'var_args', or 'keyword_args'"
            )
        } else {
            Ok(s.into())
        }
    }
}

impl Validator for ArgumentsValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        // lax_dict because mappings are always allowed to arguments
        if let Ok(dict) = input.lax_dict() {
            let val_result = match self.keyword_args {
                Some(ref kwargs_validator) => kwargs_validator.validate_generic_mapping(py, dict, input, extra, slots, recursion_guard),
                None => Err(ValError::new(ErrorKind::UnexpectedKeywordArguments, input)),
            };
            if self.positional_only_args.is_some() || self.var_args.is_some() {
                match val_result {
                    Ok(_) => Err(ValError::new(ErrorKind::MissingArguments, input)),
                    Err(val_error) => match val_error {
                        ValError::LineErrors(mut line_errors) => {
                            line_errors.push(ValLineError::new(ErrorKind::MissingArguments, input));
                            Err(ValError::LineErrors(line_errors))
                        },
                        _ => Err(val_error),
                    }
                }
            } else {
                val_result
            }
        } else {
            let tuple = input.strict_tuple()?;
            if tuple.generic_len() != 2 {
                return Err(ValError::new(ErrorKind::TwoArgumentsRequired, input));
            }
            match tuple {
                GenericListLike::Tuple(list_like) => {
                    let (args, kwargs): (&PyAny, &PyAny) = list_like.extract()?;
                },
                GenericListLike::JsonArray(list_like) => iter!(list_like),
                _ => unreachable!(),
            }
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl ArgumentsValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        args: Option<impl Input<'data>>,
        kwargs: Option<GenericMapping>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let result = match (kwargs, &self.keyword_args) {
            (Some(kwarg_dict), Some(kwargs_validator)) => {
                Some(kwargs_validator.validate_generic_mapping(py, kwarg_dict, input, extra, slots, recursion_guard))
            }
            (Some(_), None) => {
                Some(Err(ValError::new(ErrorKind::UnexpectedKeywordArguments, input)))
            }
            (None, Some(_)) => {
                Some(Err(ValError::new(ErrorKind::MissingKeywordArguments, input)))
            }
            (None, None) => None
        };
        let val_result = match self.keyword_args {
            Some(ref kwargs_validator) => kwargs_validator.validate_generic_mapping(py, dict, input, extra, slots, recursion_guard),
            None => Err(ValError::new(ErrorKind::UnexpectedKeywordArguments, input)),
        };
        if self.positional_only_args.is_some() || self.var_args.is_some() {
            match val_result {
                Ok(_) => Err(ValError::new(ErrorKind::MissingArguments, input)),
                Err(val_error) => match val_error {
                    ValError::LineErrors(mut line_errors) => {
                        line_errors.push(ValLineError::new(ErrorKind::MissingArguments, input));
                        Err(ValError::LineErrors(line_errors))
                    },
                    _ => Err(val_error),
                }
            }
        } else {
            val_result
        }
    }

}
