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


def test_validate_assignment_functions():
    calls = []

    def func_a(input_value, **kwargs):
        calls.append('func_a')
        return input_value * 2

    def func_b(input_value, **kwargs):
        calls.append('func_b')
        return input_value / 2

    v = SchemaValidator(
        {
            'type': 'model',
            'fields': {
                'field_a': {'type': 'function-after', 'function': func_a, 'field': {'type': 'str'}},
                'field_b': {'type': 'function-after', 'function': func_b, 'field': {'type': 'int'}},
            },
        }
    )

    assert v.run({'field_a': 'test', 'field_b': 12.0}) == (
        {'field_a': 'testtest', 'field_b': 6},
        {'field_a', 'field_b'},
    )

    assert calls == ['func_a', 'func_b']
    calls.clear()

    assert v.run_assignment('field_a', 'new-val', {'field_a': 'testtest', 'field_b': 6}) == (
        {'field_a': 'new-valnew-val', 'field_b': 6},
        {'field_a'},
    )
    assert calls == ['func_a']
