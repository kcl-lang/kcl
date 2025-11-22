isProdEnv = False
data = {
    env = "dev"
    if isProdEnv:
        if isProdEnv:
            env = "prod"
    else:
        env = "staging"
        staging = True
    metadata.env = env
}
