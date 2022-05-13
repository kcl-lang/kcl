c = [
    container | {
        volumeMounts = [
            {} for mount in [{"container" = "c"}] if mount.container == container.name
        ]
    } for container in [{"name" = "main"}]
]
