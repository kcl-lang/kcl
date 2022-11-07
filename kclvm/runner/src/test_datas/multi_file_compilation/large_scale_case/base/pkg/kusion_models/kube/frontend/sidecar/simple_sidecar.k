schema SimpleSidecar:
    """ Simple sidecar describes the sidecar container configuration that is expected to be run on the host.

    Attributes
    ----------
    name: str, default is Undefined, required.
        The sidecar name. e.g. 'odp','kmi','antmonitor'.
    version: str, default is Undefined, required.
        The sidecar version. e.g. 'v1.2.3'.
    extInfo: {str:}, default is Undefined, optional.
        The extended information.

    Examples
    --------
    import base.pkg.kusion_models.kube.frontend.sidecar as s

    sidecar = s.SimpleSidecar {
        name = "test"
        version = "v1.2.3"
    }
    """
    name: str
    version: str
    extInfo?: {str:}
