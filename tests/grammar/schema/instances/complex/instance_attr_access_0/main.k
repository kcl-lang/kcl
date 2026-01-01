schema Resource [name, namespace]:
    apiVersion: str = "v1"
    kind: str = "ConfigMap"
    metadata: {str:str} = {
        name = "${namespace}/${name}"
        namespace = namespace
    }

configmap = Resource("my-config", "default")
instances = Resource.instances()

instance = instances[0]

output = {
    kind: instance.kind
    metadata_name: instance.metadata.name
    metadata_namespace: instance.metadata.namespace
}
