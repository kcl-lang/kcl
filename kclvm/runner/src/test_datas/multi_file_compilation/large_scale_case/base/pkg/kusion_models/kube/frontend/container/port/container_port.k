schema ContainerPort:
    """ ContainerPort represents a network port in a single container.

    Attributes
    ----------
    name: str, default is Undefined, optional.
        If specified, this must be an IANA_SVC_NAME and unique within the pod. 
        Each named port in a pod must have a unique name. 
        Name for the port that can be referred to by services.
    protocol: "TCP" | "UDP" | "SCTP", default is "TCP", required.
        A Container-level attribute.
        The protocol for port. Must be UDP, TCP or SCTP. Default is TCP.
    containerPort: int, default is Undefined, required.
        A Container-level attribute.
        The number of port to expose on the container's IP address.

    Examples
    --------
    p = ContainerPort {
        name = "test"
        protocol = "TCP"
        containerPort = 8080
    }
    """

    name?: str
    protocol: "TCP" | "UDP" | "SCTP" = "TCP"
    containerPort: int

    check:
        1 <= containerPort <= 65535, "containerPort must be between 1 and 65535, inclusive"
