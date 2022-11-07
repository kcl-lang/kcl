schema Probe:
    """ Probe describes a health check to be performed against a container 
    to determine whether it is alive or ready to receive traffic.

    Attributes
    ----------
    handler: Exec|Http|Tcp, default is Undefined, required.
        A Container-level attribute.
        The action taken to determine the health of a container.
    initialDelaySeconds: int, default is Undefined, optional.
        A Container-level attribute.
        The length of time before health checking is activated.  In seconds.
    timeoutSeconds: int, default is Undefined, optional.
        A Container-level attribute.
        The length of time before health checking times out.  In seconds.
    periodSeconds: int, default is 10, optional.
        A Container-level attribute.
        How often (in seconds) to perform the probe.
    successThreshold: int, default is Undefined, optional.
        A Container-level attribute.
        Minimum consecutive successes for the probe to be considered successful after having failed.
    failureThreshold: int, default is Undefined, optional.
        A Container-level attribute.
        Minimum consecutive failures for the probe to be considered failed after having succeeded.

    Examples
    --------
    import base.pkg.kusion_models.kube.frontend.container.probe as p

    probe = p.Probe {
        handler = p.Http {
            path = "/healthz"
        }
        initialDelaySeconds = 10
    }
    """

    handler: Exec | Http | Tcp

    initialDelaySeconds?: int
    timeoutSeconds?: int
    periodSeconds?: int = 10
    successThreshold?: int
    failureThreshold?: int

    check:
        initialDelaySeconds >= 0 if initialDelaySeconds, "initialDelaySeconds must be greater than or equal to 0"
        timeoutSeconds >= 0 if timeoutSeconds, "timeoutSeconds must be greater than or equal to 0"
        periodSeconds >= 0 if periodSeconds, "periodSeconds must be greater than or equal to 0"
        successThreshold >= 0 if successThreshold, "successThreshold must be greater than or equal to 0"
        failureThreshold >= 0 if failureThreshold, "failureThreshold must be greater than or equal to 0"

schema Exec:
    """ Exec describes a "run in container" action. 

    Attributes
    ----------
    command: [str], default is Undefined, required.
        A Container-level attribute.
        The command line to execute inside the container.
    """

    command: [str]

    check:
        len(command) > 0, "command must be specified"

schema Http:
    """ Http describes an action based on HTTP Get requests.

    Attributes
    ----------
    path: str, default is Undefined, required.
        A Container-level attribute.
        The Path to access on the HTTP server. e.g /healthz
    port: int, default is Undefined, required.
        A Container-level attribute.
        The Number of the port to access on the container.
    scheme: "HTTP" | "HTTPS", default is "HTTP", required.
        A Container-level attribute.
        Scheme to use for connecting to the host, defaults to HTTP.
    """

    path: str
    port: int
    scheme: "HTTP" | "HTTPS" = "HTTP"

    check:
        1 <= port <= 65535, "http port must be between 1 and 65535, inclusive"

schema Tcp:
    """ Tcp describes an action based on opening a socket.

    Attributes
    ----------
    tcpSocket: int, default is Undefined, required.
        A Container-level attribute.
        The TCP socket port to connect to.
    """

    tcpSocket: int

    check:
        tcpSocket >= 1 and tcpSocket <= 65535, "tcpSocket must be between 1 and 65535, inclusive"
