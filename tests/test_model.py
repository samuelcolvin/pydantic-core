import pytest

from pydantic_core import SchemaValidator, ValidationError


def test_simple():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'type': 'str'}, 'field_b': {'type': 'int'}}})

    assert v.run({'field_a': 123, 'field_b': 1}) == ({'field_a': '123', 'field_b': 1}, {'field_b', 'field_a'})


def test_with_default():
    v = SchemaValidator(
        {'type': 'model', 'fields': {'field_a': {'type': 'str'}, 'field_b': {'type': 'int', 'default': 666}}}
    )

    assert v.run({'field_a': 123}) == ({'field_a': '123', 'field_b': 666}, {'field_a'})
    assert v.run({'field_a': 123, 'field_b': 1}) == ({'field_a': '123', 'field_b': 1}, {'field_b', 'field_a'})


def test_missing_error():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'type': 'str'}, 'field_b': {'type': 'int'}}})
    with pytest.raises(ValidationError, match='field_b | Missing data for required field'):
        v.run({'field_a': 123})


def test_ignore_extra():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'type': 'str'}, 'field_b': {'type': 'int'}}})

    assert v.run({'field_a': 123, 'field_b': 1, 'field_c': 123}) == (
        {'field_a': '123', 'field_b': 1},
        {'field_b', 'field_a'},
    )


def test_forbid_extra():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'type': 'str'}}, 'config': {'extra': 'forbid'}})

    with pytest.raises(ValidationError, match='field_b | Extra values are not permitted'):
        v.run({'field_a': 123, 'field_b': 1})


def test_allow_extra():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'type': 'str'}}, 'config': {'extra': 'allow'}})

    assert v.run({'field_a': 123, 'field_b': (1, 2)}) == ({'field_a': '123', 'field_b': (1, 2)}, {'field_a', 'field_b'})


def test_str_config():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'type': 'str'}}, 'config': {'str_max_length': 5}})
    assert v.run({'field_a': 'test'}) == ({'field_a': 'test'}, {'field_a'})

    with pytest.raises(ValidationError, match='String must have at most 5 characters'):
        v.run({'field_a': 'test long'})


def test_validate_assignment():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'type': 'str'}}})

    assert v.run({'field_a': 'test'}) == ({'field_a': 'test'}, {'field_a'})

    assert v.run_assignment('field_a', 456, {'field_a': 'test'}) == ({'field_a': '456'}, {'field_a'})