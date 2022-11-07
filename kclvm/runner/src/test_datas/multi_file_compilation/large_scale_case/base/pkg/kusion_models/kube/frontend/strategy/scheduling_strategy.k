import base.pkg.kusion_models.kube.frontend.resource as res

schema SchedulingStrategy:
    """ SchedulingStrategy represents scheduling strategy.

    Attributes
    ----------
    resource: str | res.Resource, default is Undefined, optional.
        A Pod-level attribute.
        Main container resource.
    """
    resource?: str | res.Resource
