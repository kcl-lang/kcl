schema Config[data]:
    name: str = "appName"
    globalAppName: str = name
    affinity?: {str:} = {
        "podAntiAffinity": {
            "preferredDuringSchedulingIgnoredDuringExecution": [
                {
                    "weight": 100,
                    "podAffinityTerm": {
                        "labelSelector": {
                            "matchExpressions": [
                                {
                                    "key": "cluster.k8s/app-name",
                                    "operator": "In",
                                    "values": [
                                        globalAppName
                                    ]
                                }
                            ]
                        },
                        "topologyKey": "kubernetes.io/hostname"
                    }
                }
            ]
        }
    } if data and data.antiSelf else None

config = [Config({"antiSelf": True}) {}, Config(None) {}, Config({"antiSelf": False}) {}]
