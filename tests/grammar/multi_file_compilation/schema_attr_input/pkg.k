# Stack - second file
appConfiguration: Server {
    image = "nginx:latest"
}

result = ServerBackend(inputConfig=appConfiguration).workloadAttributes