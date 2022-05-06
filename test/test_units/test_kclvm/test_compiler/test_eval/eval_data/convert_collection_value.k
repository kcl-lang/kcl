schema Node:
    name: str
    
schema AdvancedConfig:
    id?: int

schema Config:
    nodes: [Node]
    advancedConfig: AdvancedConfig

config = Config {
    nodes: [
        {name: "node1"}
        {name: "node2"}
    ]
    advancedConfig: {
        id: 1
    }
}
