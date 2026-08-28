# Case C: index-signature default. `WithComponents` has an index signature
# `[name: str]: WithComponent = {name: name}` whose default expression is
# not visible to the resolver. The static check should conservatively skip
# the inner `WithComponent {}` (which is missing `name`) because the parent
# frame has an index signature. The runtime check covers this case.
schema WithComponent:
    name: str
    $type?: str

schema WithComponents:
    [name: str]: WithComponent = {name: name}

components: WithComponents {
    ws: WithComponent {}
}