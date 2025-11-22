import ..apps

schema Server:
    name?: str
    workloadType?: str
    replicas?: int = 1
    pod?: {str:}
    deploymentModel?: apps.Deployment = _deploymentModel

    _metadata = {
        "name": name
    }
    _podTemplateSpec = {
        "metadata": {
            "name": name
        },
        "spec": pod
    }
    if workloadType == "Deployment":
        _deploymentModel = apps.Deployment {
            "metadata": _metadata,
            "spec": {
                "replicas": replicas,
                "template": _podTemplateSpec
            }
        }
