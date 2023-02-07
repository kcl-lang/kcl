isProdEnv = True
data = {
    env = "dev"
    if isProdEnv:
        env = "prod"
    metadata.env = env
}
