import pkg

config = pkg.Config {
    metadata = {
        name = "config"
    }
    spec = {
        apps = [
            {
                spec = {
                    appName = "app"
                    appType = "type"
                }
            }
        ]
    }
}
