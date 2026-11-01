schema Child:
    value: int = 1

schema Parent:
    inherited: int = 2
    optional?: str
    _hidden: int = 3

mixin FieldsMixin:
    mixed = 4
    combine = lambda x: int {
        x + mixed
    }

schema Data(Parent):
    mixin [FieldsMixin]
    mixed: int
    combine: (int) -> int
    child: Child = Child {}
    measure: any = len
    instances: str = "attribute"

schema Relaxed:
    [str]: int
    defaulted: int = 5

_default: any = Data {}
_configured: any = Data {
    inherited = 20
    optional = Undefined
    _hidden = 30
    child.value = 10
}
_holder = {value = _configured}
_relaxed: any = Relaxed {extra = 6}

inherited_default = _default.inherited
inherited_config = _configured.inherited
hidden_default = _default._hidden
hidden_config = _configured._hidden
mixed = _configured.mixed
mixed_method = _holder.value.combine(1)
builtin = _configured.measure([1, 2])
instances_attr = _configured.instances
optional_is_undefined = _configured.optional == Undefined
nested_schema = typeof(_configured.child, full_name=True)
nested_default = _default.child.value
nested_config = _configured.child.value
relaxed_default = _relaxed.defaulted
relaxed_config = _relaxed.extra
