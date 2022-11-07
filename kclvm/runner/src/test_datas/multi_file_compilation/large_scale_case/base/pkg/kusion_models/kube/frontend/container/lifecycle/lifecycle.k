import base.pkg.kusion_models.kube.frontend.container.probe as p

schema Lifecycle:
    """ Lifecycle describes actions that the management system should take in response 
    to container lifecycle events.

    Attributes
    ----------
    preStop: p.Exec|p.Http, default is Undefined, optional.
        A Container-level attribute.
        The PreStop action is called immediately before a container is terminated.
    postStart: p.Exec|p.Http, default is Undefined, optional.
        A Container-level attribute.
        The PostStart action is called immediately after a container is created.

    Examples
    --------
    import base.pkg.kusion_models.kube.frontend.container.lifecycle as lc
    import base.pkg.kusion_models.kube.frontend.container.probe as p

    p = lc.Lifecycle {
        preStop = p.Exec {
            command = [
                "timeout"
                "--signal=9"
                "1800s"
                "sh"
                "-c"
                "bash -x /tmp/image-builder/boot/boot.sh"
            ]
        }
    }
    """

    preStop?: p.Exec | p.Http
    postStart?: p.Exec | p.Http
