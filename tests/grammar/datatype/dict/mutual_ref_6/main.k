isProdEnv = False
data = {
    env = "dev"
    if isProdEnv:
        env = "prod"
    else:
        env = "staging"
    metadata.env = env
}
