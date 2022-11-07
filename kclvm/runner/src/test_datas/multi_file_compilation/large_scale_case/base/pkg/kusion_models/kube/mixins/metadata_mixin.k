mixin MetadataMixin:
    metadata?: {str:} = {
        name: name?.lower()
        annotations: annotations
        namespace: namespace
        labels: labels
    }