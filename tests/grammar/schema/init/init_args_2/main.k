schema Backend:
    metadata: {str:}
    spec: {str:}

schema Wrapper[data]:
    kubeResource: Backend = Backend {
        metadata: {
            "name": data["name"]
        },
        spec: {
            podSpec: {
                template: {
                    containers: [{
                        image: data["image"]
                    }]
                }
            }
        }
    }

_front = {
    "name": "front",
    "image": "docker:xx"
}

back = Wrapper(_front) {}
