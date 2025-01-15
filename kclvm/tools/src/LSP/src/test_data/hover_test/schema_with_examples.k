schema Server:
    """Server is an abstraction of Deployment and StatefulSet.

    Attributes
    ----------
    workloadType : str, default is "Deployment", required
        Use this attribute to specify which kind of long-running service you want.
        Valid values: Deployment, StatefulSet.
    name : str, required
        The name of the long-running service.
    labels : {str:str}, optional
        The labels of the long-running service.

    Examples
    --------
    import models.kube.frontend
    import models.kube.frontend.container
    import models.kube.templates.resource as res_tpl
    
    appConfiguration: frontend.Server {
        mainContainer = container.Main {
            name = "php-redis"
            env: {
                "GET_HOSTS_FROM": {value = "dns"}
            }
        }
    }
    """
    workloadType : str = "Deployment"
    name: str
    labels?: {str: str}
