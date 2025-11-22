schema WithComponent:
    name: str
    $type?: str

schema WithComponents:
    [name: str]: WithComponent = {name: name}

components: WithComponents {
    ws: {}
}
