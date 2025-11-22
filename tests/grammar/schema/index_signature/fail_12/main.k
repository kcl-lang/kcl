type _foo = "foo" | "bar"

schema IndexSignature:
    [_foo]: str

IndexSignature {
    foo = "foo"
    # this should throw an error
    baz: "baz"
}
