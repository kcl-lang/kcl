import ..apicore

schema DeploymentSpec:
    template?: apicore.PodTemplateSpec
    replicas?: int
