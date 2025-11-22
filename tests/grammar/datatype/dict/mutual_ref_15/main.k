render = lambda {
    a = {
        foo: "bar"
    }
    b = {
        foo2: "bar2"
        a: {
            b: "c"
        }
    }
    c = [a, b]
}

out = render()
a = {
    foo: "bar"
}
b = {
    foo2: "bar2"
    a: {
        b: "c"
    }
}
c = [a, b]
