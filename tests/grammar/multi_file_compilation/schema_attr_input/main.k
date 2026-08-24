schema Server:
    replicas: int = option("replicas") or 1
    image: str
    mainContainer?: any

schema ServerBackend[inputConfig: Server]:
    config: Server = inputConfig
    app: any = None
    workloadAttributes: {str:} = {
        replicas = config.replicas
        name = app
    }

# Base config provided by the main file; pkg.k consumes it.
appConfiguration: Server {
    replicas = 3
    image = "nginx:1.7.8"
}