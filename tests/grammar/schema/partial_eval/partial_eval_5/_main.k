schema EmptyDir:
    medium: "" | "Memory"
    sizeLimit?: str

schema Config:
    config: {str:} = {
        volumes = [
            {
                name = "kubeconfig"
                volumeSource = EmptyDir {medium = ""}
            }
            {
                name = "log-storage"
                volumeSource = EmptyDir {medium = "Memory"}
            }
        ]
    }

    workload: {str:} = {
        spec = {
            template = {
                spec = {
                    if config.volumes: volumes = [
                        (lambda volume {
                            {
                                name = volume.name
                            }
                        })(v) for v in config.volumes if v.volumeSource
                    ]
                }
            }
        }
    }

config = Config {}
