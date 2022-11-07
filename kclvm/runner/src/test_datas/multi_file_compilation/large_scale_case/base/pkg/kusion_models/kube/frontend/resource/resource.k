import units

type Unit = units.NumberMultiplier

schema Resource:
    """ Resource describes the compute resource requirements.

    Attributes
    ----------
    cpu: int | Unit, default is 1, optional.
        A Container-level attribute.
        CPU, in cores, default 1 core. (500m = .5 cores)
    memory: Unit, default is 1024Mi, optional.
        A Container-level attribute.
        Memory, in bytes, default 1024Mi. (500Gi = 500GiB = 500 * 1024 * 1024 * 1024)
    disk: Unit, default is 10Gi, optional.
        A Container-level attribute.
        Local disk storage, in bytes, default 10Gi. (500Gi = 500GiB = 500 * 1024 * 1024 * 1024)

    Examples
    --------
    import base.pkg.kusion_models.kube.frontend.resource as res

    res = res.Resource {
        cpu = 2
        memory = 2048Mi
        disk = 20Gi
    }
    """

    cpu?: int | Unit = 1
    memory?: Unit = 1024Mi
    disk?: Unit = 10Gi
