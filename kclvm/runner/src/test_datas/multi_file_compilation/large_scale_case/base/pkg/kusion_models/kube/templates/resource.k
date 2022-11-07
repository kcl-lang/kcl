import base.pkg.kusion_models.kube.frontend.resource as res

tiny = res.Resource {
    cpu = 100m
    memory = 100Mi
    disk = 1Gi
}

small = res.Resource {
    cpu = 1
    memory = 2Gi
    disk = 20Gi
}

medium = res.Resource {
    cpu = 2
    memory = 4Gi
    disk = 35Gi
}

medium100G = res.Resource {
    cpu = 2
    memory = 4Gi
    disk = 100Gi
}

large = res.Resource {
    cpu = 4
    memory = 8Gi
    disk = 70Gi
}

large100G = res.Resource {
    cpu = 4
    memory = 8Gi
    disk = 100Gi
}

large2X32G40G = res.Resource {
    cpu = 8
    memory = 32Gi
    disk = 40Gi
}

large4X = res.Resource {
    cpu = 16
    memory = 32Gi
    disk = 20Gi
}
