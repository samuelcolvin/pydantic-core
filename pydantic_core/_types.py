from __future__ import annotations

from typing import Any, Dict, List, Literal, Sequence, TypedDict, Union

from typing_extensions import NotRequired


class AnySchema(TypedDict):
    type: Literal['any']


class BoolSchema(TypedDict):
    type: Literal['bool']
    strict: NotRequired[bool]


class ConfigSchema(TypedDict):
    strict: NotRequired[bool]
    extra: NotRequired[Literal['allow', 'forbid', 'ignore']]


class DictSchema(TypedDict):
    type: Literal['dict']
    keys: Schema  # type: ignore[misc]
    values: Schema  # type: ignore[misc]
    min_items: NotRequired[int]
    max_items: NotRequired[int]


class FloatSchema(TypedDict):
    type: Literal['float']
    multiple_of: NotRequired[float]
    le: NotRequired[float]
    ge: NotRequired[float]
    lt: NotRequired[float]
    gt: NotRequired[float]
    strict: NotRequired[bool]
    default: NotRequired[float]


class FunctionSchema(TypedDict):
    type: Literal['function']
    mode: Literal['before', 'after', 'plain', 'wrap']


class IntSchema(TypedDict):
    type: Literal['int']
    multiple_of: NotRequired[int]
    le: NotRequired[int]
    ge: NotRequired[int]
    lt: NotRequired[int]
    gt: NotRequired[int]
    strict: NotRequired[bool]


class ListSchema(TypedDict):
    type: Literal['list']
    items: Schema  # type: ignore[misc]
    min_items: NotRequired[int]
    max_items: NotRequired[int]


class LiteralSchema(TypedDict):
    type: Literal['literal']
    expected: Sequence[Any]


class ModelClassSchema(TypedDict):
    type: Literal['model_class']
    class_type: type
    model: ModelSchema  # type: ignore[misc]


class ModelSchema(TypedDict):
    type: Literal['model']
    fields: Dict[str, Schema]  # type: ignore[misc]
    name: NotRequired[str]
    extra_validator: NotRequired[Schema]  # type: ignore[misc]
    config: NotRequired[ConfigSchema]


class NoneSchema(TypedDict):
    type: Literal['none']


class OptionalSchema(TypedDict):
    type: Literal['optional']
    schema: Schema  # type: ignore[misc]
    strict: NotRequired[bool]


class RecursiveReferenceSchema(TypedDict):
    type: Literal['recursive-ref']
    name: str


class RecursiveContainerSchema(TypedDict):
    type: Literal['recursive-container']
    name: str
    schema: Schema  # type: ignore[misc]


class SetSchema(TypedDict):
    type: Literal['set']
    items: Schema  # type: ignore[misc]
    min_items: NotRequired[int]
    max_items: NotRequired[int]
    strict: NotRequired[bool]


class StringSchema(TypedDict, total=False):
    type: Literal['str']
    pattern: NotRequired[str]
    max_length: NotRequired[int]
    min_length: NotRequired[int]
    strip_whitespace: NotRequired[bool]
    to_lower: NotRequired[bool]
    to_upper: NotRequired[bool]
    strict: NotRequired[bool]


class UnionSchema(TypedDict):
    type: Literal['union']
    choices: List[Schema]  # type: ignore[misc]
    strict: NotRequired[bool]


Type = str

Schema = Union[  # type: ignore[misc]
    Type,  # bare type,  'str'
    AnySchema,
    BoolSchema,
    DictSchema,
    FloatSchema,
    FunctionSchema,
    IntSchema,
    ListSchema,
    LiteralSchema,
    ModelSchema,
    ModelClassSchema,
    NoneSchema,
    OptionalSchema,
    RecursiveContainerSchema,
    RecursiveReferenceSchema,
    SetSchema,
    StringSchema,
    UnionSchema,
]