schema ObjectMeta:
    name?: str

schema EmptyDirVolumeSource:
    medium?: str
    sizeLimit?: str

schema Volume:
    name?: str
    emptyDir?: EmptyDirVolumeSource

schema PodSpec:
    volumes?: [Volume]
    activeDeadlineSeconds?: int

schema PodTemplateSpec:
    metadata?: ObjectMeta
    spec?: PodSpec

schema Deployment:
    apiVersion?: str = "apps/v1"
    kind?: str = "Deployment"
    spec?: DeploymentSpec
    metadata?: ObjectMeta

schema DeploymentSpec:
    template?: PodTemplateSpec
    replicas?: int

schema Server:
    name?: str
    workloadType?: str
    replicas?: int = 1
    pod?: {str:}
    deploymentModel?: Deployment = _deploymentModel

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
        _deploymentModel = Deployment {
            "metadata": _metadata,
            "spec": {
                "replicas": replicas,
                "template": _podTemplateSpec
            }
        }

server = Server {
    "name": "productPage",
    "workloadType": "Deployment",
    "pod": {
        "volumes": [{"name": "tmp", "emptyDir": {}}],
        "activeDeadlineSeconds": 600
    }
}
