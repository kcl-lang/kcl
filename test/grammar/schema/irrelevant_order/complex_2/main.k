schema KubeNamespace:
    __settings__: {str:str} = {"output_type": "STANDALONE"}
    apiVersion: str = "v1"
    kind: str = "Namespace"
    metadata: {str:}
    spec: {str:}

schema MetadataMixin:
    metadata: {str:} = {
        "name": name,
        "annotations": annotations,
        "namespace": namespace,
        "labels": labels
    }

schema NamespaceMixin:
    _spec: {str:} = {"finalizers": finalizers} if finalizers else None
    kubeNamespace: KubeNamespace = KubeNamespace {
        metadata: metadata,
        spec: _spec
    }

schema Metadata:
    mixin [MetadataMixin]

    name: str
    labels?: {str:str}
    annotations?: {str:str}
    namespace?: str

schema Namespace(Metadata):
    mixin [NamespaceMixin]

    name: str = "prod-frontend"
    finalizers: [str]

    __settings__: {str:} = {
        "output_type": "IGNORE"
    }

node = Namespace {
    finalizers: ["test"]
}
