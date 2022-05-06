x = (lambda {
    if True:
        _x = False
    else:
        _x = True
    _x
})()
y = (lambda {
    _x = True
    if True:
        _x = False
    _x
})()
