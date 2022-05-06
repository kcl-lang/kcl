import ..apimachinery

schema Deployment:
    apiVersion?: str = "apps/v1"
    kind?: str = "Deployment"
    spec?: DeploymentSpec
    metadata?: apimachinery.ObjectMeta
