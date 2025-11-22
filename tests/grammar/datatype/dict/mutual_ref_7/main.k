isProdEnv = True
data = {
    env = "dev"
    if False:
        env = "test"
    elif isProdEnv:
        env = "prod"
    else:
        env = "staging"
    metadata.env = env
}
