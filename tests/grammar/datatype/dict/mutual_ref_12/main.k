c = [
    container | {
        volumeMounts = [
            {name = mount.name} for mount in [{"container" = "main", name = container + "Container"}] if mount.container == container.name
        ]
    } for container in [{"name" = "main"}]
]
