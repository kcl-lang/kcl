import base.pkg.kusion_models.kube.frontend.common

schema ConfigMap(common.Metadata):
    """ConfigMap holds configuration data for pods to consume.
    More info: https://kubernetes.io/docs/reference/kubernetes-api/config-and-storage-resources/config-map-v1/#ConfigMap
    
    Attributes
    ----------
    data: {str:str}, default is Undefined, optional.
        Data contains the configuration data.
    binaryData: {str:str}, default is Undefined, optional
        BinaryData contains the binary data.
    
    Examples
    --------
    configmap = ConfigMap {
        name = "my-configmap"
        namespace = "my-configmap-namespace"
        data = {
            foo = "bar"
            bar = "foo"
        }
    }
    """
    data?: {str:str}
    binaryData?: {str:str}
