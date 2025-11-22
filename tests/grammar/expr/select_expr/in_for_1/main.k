c = [
    container | {
        volumeMounts = [
            {} for mount in [{"container" = {"name" = "main"}}] if mount.container.name == container.name
        ]
    } for container in [{"name" = "main"}]
]
