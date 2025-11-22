schema Backend:
    metadata?: {str:}
    spec?: {str:}
    key?: str

schema Wrapper[data: Frontend]:
    ok?: str = "123"
    dataOn?: Frontend = Frontend {
        name: "str"
    }
    kubeResource?: Backend = Backend {
        metadata: {
            "name": data.name_err_key,
            "spec": dataOn.name,
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

schema Frontend:
    name?: str
    image?: str

_front = Frontend {
    name: "front",
    image: "docker:xx"
}
back = Wrapper(_front) {}
