import base.pkg.kusion_models.kube.metadata

MetadataBuilder = lambda config -> {str:} {
    {
        name: metadata.__META_APP_NAME
        namespace: metadata.__META_APP_NAME
        labels: config?.labels
        annotations: config?.annotations
    }
}
