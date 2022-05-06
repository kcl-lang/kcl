schema KubeAction:
    __settings__: {str:str} = {"output_type": "STANDALONE"}
    apiVersion: str = "kusion/v1"
    kind: str = "Action"
    metadata: {str:}
    spec: {str:}

schema Action:
    __settings__: {str:str} = {"output_type": "IGNORE"}
    name: str
    pod: str
    namespace: str
    cluster: str
    type: str

    kubeAction: KubeAction = KubeAction {
        metadata.name: name
        spec: {
            pod: pod,
            namespace: namespace,
            cluster: cluster,
            type: type
        }
    }


schema KeyValueUpdateAction(Action):
    keyValues: {str:}
    query: {str:}
    # kubeAction: KubeAction
    pod = "podname"
    namespace = "namespace"
    cluster: str = "cluster"
    type: str = "pods"
    name: str = "pod_key_value_update"

    kubeAction.spec |= {
        "keyValues": keyValues
    }

act = KeyValueUpdateAction() {
    query.hostnames: ["foo.bar"]
    keyValues.key: "val"
}
