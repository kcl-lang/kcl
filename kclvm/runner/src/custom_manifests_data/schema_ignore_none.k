import manifests

schema Person:
    name: str = "kcl"
    age?: int = 1

x0 = Person {}
x1 = Person {
    age = None
}
manifests.yaml_stream([x0, x1], opts = {ignore_none = True})
