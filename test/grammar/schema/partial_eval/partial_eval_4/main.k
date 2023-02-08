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

config = Config {}
