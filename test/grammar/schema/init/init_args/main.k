schema Backend:
    metadata?: {str:}
    spec?: {str:}
    key?: str = None

schema Frontend:
    name?: str = None
    image?: str = None

schema Wrapper[data]:
    ok?: str = "123"
    dataOn?: Frontend = Frontend {
        name: "str"
    }
    kubeResource?: Backend = Backend {
        metadata: {
            "name": data.name,
            spec: dataOn.name,
            key: data.name
        },
        spec: {
            podSpec: {
                template: {
                    containers: [{
                        image: ok
                    }]
                }
            }
        }
    }

_front = Frontend {
    name: "front",
    image: "docker:xx"
}
back = Wrapper(_front) {}
