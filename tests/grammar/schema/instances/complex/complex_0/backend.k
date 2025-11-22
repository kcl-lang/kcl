schema Backend:
    name?: str
    metadata?: {str:}

backends = [Backend {
    name: model.name
    metadata.labels: model.labels
    metadata.annotations: model.annotations
} for model in Model.instances()]  
