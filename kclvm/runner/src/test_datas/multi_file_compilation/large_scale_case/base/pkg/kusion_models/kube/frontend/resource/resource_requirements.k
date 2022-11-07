schema ResourceRequirements:
    """ ResourceRequirements describes the compute resource requirements..

    Attributes
    ----------
    limits: Resource, default is Undefined, required.
        A Container-level attribute.
        Limits describes the maximum amount of compute resources allowed. 
        More info: https://kubernetes.io/docs/concepts/configuration/manage-compute-resources-container/
    requests: Resource, default is Undefined, required.
        A Container-level attribute.
        Requests describes the minimum amount of compute resources required. 
        If Requests is omitted for a container, it defaults to Limits if that is explicitly specified, otherwise to an implementation-defined value. 
        More info: https://kubernetes.io/docs/concepts/configuration/manage-compute-resources-container/

    Examples
    --------
    import base.pkg.kusion_models.kube.frontend.resource as res

    res = res.ResourceRequirements {
        limits = {
            cpu = 1
            memory = 1Gi
            disk = 20Gi
        }
        requests = {
            cpu = 500m
            memory = 512Mi
            disk = 10Gi
        }
    }
    """

    limits: Resource
    requests: Resource
