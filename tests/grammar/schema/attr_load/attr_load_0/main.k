schema Child:
    value: int = 1

schema Data:
    name: str
    count: int = 0
    nick?: str
    alias?: str = None
    child: Child = Child {}
    labels: {str:str} = {}
    add = lambda x: int {
        x + count
    }

    check:
        count >= 0

schema Relaxed:
    name: str
    [...str]: str

_data = Data {name = "data", count = 2}
_any: any = _data
_relaxed: any = Relaxed {name = "relaxed", extra = "x"}
_none: any = None

name = _any.name
count = _any.count
nick_is_undefined = _any.nick == Undefined
alias_is_none = _any.alias == None
child_value = _any.child.value
labels = _any.labels
added = _any.add(1)
extra = _relaxed.extra
opt_name = _any?.name
opt_none = _none?.name
counts = [_any.count for _ in range(3)]
data = _data
