schema Backend:
    __settings__: {str:str} = {"output_type": "STANDALONE"}
    """ existence of this attribute indicates that the model will be treated standalone by KCLVM.
    """

    apiVersion?: str = "v1"
    kind?: str = "Deployment"
    metadata?: {str:}
    spec?: {str:} = {
        "minReadySeconds" = 0
        "paused" = False
        "progressDeadlineSeconds" = 600
        "replicas" = 1
        "revisionHistoryLimit" = 10
        "selector": {}
    }

backends = [Backend {
    metadata.name: model.name
    metadata.labels: model.labels
    metadata.annotations: model.annotations
    spec.selector.matchLabels: model.labels
    spec.replicas = model.replicas
} for model in Model.instances()]  
