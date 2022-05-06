from kclvm.api.object import (
    KCLObject,
    KCLStringObject,
    KCLListObject,
    KCLDictObject,
    KCLSchemaObject,
    to_python_obj,
    to_kcl_obj,
)


def plus(left: KCLObject, right: KCLObject):
    if isinstance(left, KCLStringObject) and isinstance(right, KCLStringObject):
        return KCLStringObject(value=left.value + right.value)
    if isinstance(left, KCLListObject) and isinstance(right, KCLListObject):
        return KCLListObject(items=left.items + right.items)
    return to_kcl_obj(to_python_obj(left) + to_python_obj(right))


def handle_subscript(obj: KCLObject, slice: KCLObject):
    return to_kcl_obj(
        obj.get(to_python_obj(slice))
        if isinstance(obj, (KCLDictObject, KCLSchemaObject))
        else obj.value[to_python_obj(slice)]
    )
