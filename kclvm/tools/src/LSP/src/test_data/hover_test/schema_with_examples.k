schema Server:
    """Server is abstaction of Deployment and StatefulSet.
    
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
    name?: str