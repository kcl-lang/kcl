schema Server:
    replicas: int = option("replicas") or 1
    image: str
    mainContainer?: any

schema ServerBackend[inputConfig: Server]:
    config: Server = inputConfig
    app: any = None
    workloadAttributes: {str:} = {
        spec = {
            replicas = config.replicas
        }
    }

# Base definition - first file
appConfiguration: Server {
    image = "nginx:1.7.8"
    mainContainer = {}
}