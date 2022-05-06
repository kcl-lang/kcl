schema Container:
    """A single application container that you want to run within a pod

    Attributes
    ----------
    image : str, Default is Undefined, required
        Docker image name.
    ports: [ContainerPort], Default is Undefined, optional
        List of ports to expose from the container.
    """
    image: str
    ports?: [ContainerPort]


schema ContainerPort:
    """ContainerPort represents a network port in a single container.

    Attributes
    ----------
    port : int, Default is Undefined, required
        Number of port to expose on the pod's IP address. This must be a valid port number, 0 < x < 65536.
    protocol : "TCP" | "UDP" | "SCTP", Default is "TCP", required
        Protocol for port. Must be UDP, TCP, or SCTP.
    """
    port: int
    protocol: "TCP" | "UDP" | "SCTP" = "TCP"

    check:
        0 < port < 65536


