schema WithComponent:
    name?: str
    $type?: str

schema WithComponents:
    [name: str]: WithComponent = WithComponent {name: name}

components: WithComponents {
    ws = {}
}
