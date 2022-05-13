schema Server:
    """Server defined the classical Long-running Service.

    Attributes
    ----------
    name : str, Default is Undefined, required
        The name of the server.
    workloadType : "Deployment" | "StatefulSet" | "DaemonSet", Default is "Deployment", required
        The type of the workload.
    replica : int, Default is 1, required
        The desired replica count
    mainContainer : Container, Default is Undefined, required
        The main container.
    """
    name: str
    workloadType: "Deployment" | "StatefulSet" | "DaemonSet" = "Deployment"
    replica: int = 1
    mainContainer: Container
