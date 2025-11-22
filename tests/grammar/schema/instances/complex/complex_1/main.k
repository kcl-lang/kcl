schema Model:
    name?: str
    labels?: {str:} = None
    annotations?: {str:} = None

model1 = Model {
    name: "model1"
    labels.key1: "value1"
    labels.key2: "value2"
    annotations.key: "value"
}

model2 = Model {
    name: "model2"
}
