schema Model:
    name?: str
    labels?: {str:} = None
    annotations?: {str:} = None
    replicas?: int

_model1 = Model {
    name: "model1"
    labels.key1: "value1"
    labels.key2: "value2"
    annotations.key: "value"
    replicas = 2
}

_model2 = Model {
    name: "model2"
    replicas = 3
}
