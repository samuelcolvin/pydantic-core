use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{ErrorKind, ValError, ValLineError, ValResult};
use crate::input::{GenericListLike, GenericMapping, Input};
use crate::recursion_guard::RecursionGuard;

use super::tuple::{TuplePositionalValidator, TupleVariableValidator};
use super::typed_dict::TypedDictValidator;
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ArgumentsValidator {
    // TODO use nohash-hasher
    argument_mapping: Option<Vec<(usize, String)>>,
    positional_args: Option<TuplePositionalValidator>,
    keyword_args: Option<TypedDictValidator>,
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
        let positional_args = build_specific_validator!("positional_args", TuplePositional);
        let p_args_name = match positional_args {
            Some(ref v) => v.get_name(),
            None => "-",
        };
        let keyword_args = build_specific_validator!("keyword_args", TypedDict);
        let k_args_name = match keyword_args {
            Some(ref v) => v.get_name(),
            None => "-",
        };
        if positional_args.is_none() && keyword_args.is_none() {
            return py_error!("Arguments schema must have either 'positional_args' or 'keyword_args' defined");
        }

        Ok(Self {
            argument_mapping: None,
            positional_args,
            keyword_args,
            name: format!("{}[{}, {}]", Self::EXPECTED_TYPE, p_args_name, k_args_name),
        }
        .into())
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
            self.validate_args_kwargs(py, None, Some(dict), input, extra, slots, recursion_guard)
        } else {
            let tuple = input.strict_tuple()?;
            if tuple.generic_len() != 2 {
                return Err(ValError::new(ErrorKind::TwoArgumentsRequired, input));
            }
            match tuple {
                GenericListLike::Tuple(list_like) => {
                    let (args, kwargs): (&PyAny, &PyAny) = list_like.extract()?;
                    todo!("args: {}, kwargs: {}", args, kwargs)
                }
                GenericListLike::JsonArray(list_like) => {
                    todo!()
                }
                _ => unreachable!(),
            }
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl ArgumentsValidator {
    fn validate_args_kwargs<'s, 'data>(
        &'s self,
        py: Python<'data>,
        args: Option<GenericListLike>,
        kwargs: Option<GenericMapping>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        match (args, &self.argument_mapping) {
            (Some(args), Some(argument_mapping)) => {
                let kwargs = match kwargs {
                    Some(kwargs) => todo!(),
                    None => PyDict::new(py),
                };
                for (index, name) in argument_mapping {

                }
            }
        }

        let arg_result = match (args, &self.positional_args) {
            (Some(args), Some(args_validator)) => {
                Some(args_validator.validate_list_like(py, args, input, extra, slots, recursion_guard))
            }
            (Some(_), None) => Some(Err(ValError::new(ErrorKind::UnexpectedPositionalArguments, input))),
            (None, Some(_)) => Some(Err(ValError::new(ErrorKind::MissingPositionalArguments, input))),
            (None, None) => None,
        };
        match arg_result {
            Err(ValError::InternalErr(err)) => return Err(ValError::InternalErr(err)),
            _ => (),
        };

        let kwarg_result = match (kwargs, &self.keyword_args) {
            (Some(args), Some(kwargs_validator)) => {
                Some(kwargs_validator.validate_generic_mapping(py, args, input, extra, slots, recursion_guard))
            }
            (Some(_), None) => Some(Err(ValError::new(ErrorKind::UnexpectedKeywordArguments, input))),
            (None, Some(_)) => Some(Err(ValError::new(ErrorKind::MissingKeywordArguments, input))),
            (None, None) => None,
        };

        match (arg_result, kwarg_result) {
            (Ok(args), Ok(kwargs)) => Ok((args, kwargs).to_object(py)),
            (Err(args_error), Ok(_)) => Err(args_error),
            (Ok(_), Err(kwargs_error)) => Err(kwargs_error),
            (Err(_), Err(ValError::InternalErr(err))) => Err(ValError::InternalErr(err)),
            (Err(ValError::LineErrors(mut args_errors)), Err(ValError::LineErrors(kwargs_errors))) => {
                args_errors.extend(kwargs_errors);
                Err(ValError::LineErrors(args_errors))
            }
            _ => unreachable!(),
        }
    }
}
