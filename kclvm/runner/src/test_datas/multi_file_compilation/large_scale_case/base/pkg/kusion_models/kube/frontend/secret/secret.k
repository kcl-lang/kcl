import base.pkg.kusion_models.kube.frontend.common

schema Secret(common.Metadata):
    """Secret holds secret data of a certain type. 
    The total bytes of the values in the Data field
    must be less than MaxSecretSize bytes.

    Attributes
    ----------
    data: {str:str}, default is Undefined, optional
        Data contains the secret data. Each key must consist of alphanumeric characters, '-', '_' or '.'. 
        More info: https://kubernetes.io/docs/concepts/configuration/secret/#restriction-names-data
    stringData: {str:str}, default is Undefined, optional
        stringData allows specifying non-binary secret data in string form. 
        More info: https://kubernetes.io/docs/concepts/configuration/secret/#restriction-names-data
    type: str, default is Undefined, optional
        Used to facilitate programmatic handling of secret data.
        More info: https://kubernetes.io/docs/concepts/configuration/secret/#secret-types
    
    Examples
    --------
    secret = Secret {
        name = "my-secret"
        namespace = "my-secret-namespace"
        data = {
            foo = bar
            bar = foo
        }
        $type = "kubernetes.io/service-account-token"
    }
    """
    data?: {str:str}
    stringData?: {str:str}
    type?: str
