schema Server:
    """Server is the common user interface for long-running
    services adopting the best practice of Kubernetes.

    Attributes
    ----------
    workloadType : default is Deployment
        Use this attribute to specify which kind of long-running service you want.
        Valid values: Deployment, CafeDeployment.
        See also: kusion_models/core/v1/workload_metadata.k.
    name
        A Server-level attribute.
        The name of the long-running service.
        See also: kusion_models/core/v1/metadata.k.
    labels : optional
        A Server-level attribute.
        The labels of the long-running service.
        See also: kusion_models/core/v1/metadata.k.
    replica : default is 1, optional
        Replica of the server.

    Examples
    ----------------------
    myCustomApp = AppConfiguration {
        name: "componentName"
    }
    """
    workloadType: str = "Deployment"
    name: str
    labels?: {str: str}
    replica?: int = 1
