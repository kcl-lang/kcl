deployment = {
    metadata.labels.k1 = "v1"
    metadata.namespace = "default"
    spec.selector.matchLabels = metadata.labels | {k2 = "v2"}
}
labels: {str:str} = deployment.metadata.labels
