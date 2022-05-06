x = (lambda {
    _e = 1
    if True:
        _e = 2
    {
        e: _e
    }
})()
_e = 1
if True:
    _e = 2
y = {
    e: _e
}
