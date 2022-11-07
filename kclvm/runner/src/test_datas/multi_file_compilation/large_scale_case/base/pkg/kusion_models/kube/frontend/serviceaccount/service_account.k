import base.pkg.kusion_models.kube.frontend.common

schema ServiceAccount(common.Metadata):
    """A service account provides an identity for processes that run in a Pod.
    ServiceAccount binds together:
        - a name, understood by users, and perhaps by peripheral systems, for an identity
        - a principal that can be authenticated and authorized
        - a set of secrets
    More info: https://kubernetes.io/docs/reference/kubernetes-api/authentication-resources/service-account-v1/#ServiceAccount

    Attributes
    ----------
	imagePullSecrets: [{str:str}], default is Undefined, optional.
        ImagePullSecrets is a list of references to secrets in the same namespace to use for pulling any images in pods that reference this ServiceAccount.
        More info: https://kubernetes.io/docs/concepts/containers/images/#specifying-imagepullsecrets-on-a-pod
	secrets: [{str:str}], default is Undefined, optional.
        Secrets is the list of secrets allowed to be used by pods running using this ServiceAccount.
        More info: https://kubernetes.io/docs/concepts/configuration/secret

    Examples
    --------
    my_service_account = ServiceAccount {
        name: "my-service-account"
        namespace = "my-service-account-namespace"
        labels: {
            tier: "monitoring"
        }
        imagePullSecrets: [
            {
                name: "my-secret"
            }
        ]
        secrets: [
            {
                name: "my-secret"
            }
        ]
    }
    """

    imagePullSecrets?: [{str:str}]
    secrets?: [{str:str}]

    check:
        name, "name must be specified and can't be empty or None or Undefined"
