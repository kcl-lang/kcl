schema Outer:
    inner: Inner

schema Inner:
    name: str

    check:
        len(name) > 0

repro: Outer = {
    inner: {
        name: ""
    }
}