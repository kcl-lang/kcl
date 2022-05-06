schema app_conf:
    services: [{str:str}]

    check:
        all service in services {
            service.clusterIP == "NONE" if service.type == "ClusterIP"
        }, "invalid cluster ip"


main = app_conf {
    services: [{
        type: "ClusterIP"
        clusterIP: "NONE"
    }]
}