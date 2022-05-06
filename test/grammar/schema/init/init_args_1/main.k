schema Backend:
    metadata: {str:}
    spec: {str:}

schema Wrapper[data]:
    kubeResource: Backend = Backend {
        metadata: {
            "name": data.name
        },
        spec: {
            podSpec: {
                template: {
                    containers: [{
                        image: data.image
                    }]
                }
            }
        }
    }

schema Frontend:
    name: str
    image: str

_front = Frontend {
    name: "front",
    image: "docker:xx"
}

back = Wrapper(_front) {}
