name = "global-release-name"
data = {
    name = "data-release"
    metadata.name = name
    spec.template.metadata.name = metadata.name
}
metadata = {
    name = "metadata-release-name"
    labels = {
        "app.kubernetes.io/name" = name
        "app.kubernetes.io/instance" = name
    }
}
