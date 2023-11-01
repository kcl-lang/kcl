global = 1
func: ( int|str, int|str ) -> int = lambda x: int | str, y: int | str {
    (lambda x, y {
        int(x) + int(y) + global
    })(x, y)  
}
a = func(1, 1)
b = func("1", "1")
