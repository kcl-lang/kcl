_looper_n = lambda elements: [any], n: int, func: (any, any) -> any, initial: any-> any {
    assert n >= 0
    result = initial
    if n < len(elements):
        result = _looper_n(elements, n + 1, func, func(result, elements[n]))
    result
}

looper = lambda initial: any, elements: [any], func: (any, any) -> any -> any {
    _looper_n(elements, 0, func, initial)
}

for_each = lambda elements: [any], func: (any) -> any {
    _looper_n(elements, 0, lambda v, e {
        func(e)
    }, Undefined)
}

x = looper(0, [1, 2, 3], lambda i, x {
    i + x
})
