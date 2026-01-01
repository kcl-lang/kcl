schema Pod [name, replicas]:
    apiVersion: str = "v1"
    kind: str = "Pod"
    metadata: {str:} = {
        name = name
        labels: {
            app: name
            count: "${replicas}"
        }
    }
    spec: {str:} = {
        replicas: replicas
    }

_ = Pod("web", 3)
_ = Pod("api", 2)

instances = Pod.instances()

_first = instances[0]
_second = instances[1]

_first.kind = "Deployment"
_second.kind = "StatefulSet"

