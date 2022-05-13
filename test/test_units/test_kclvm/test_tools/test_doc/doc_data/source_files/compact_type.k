schema Metadata:
    '''Metadata is the base schema of all models, which contains data
    that helps uniquely identify the object.

    Attributes
    ----------
    name: str
        The name of the resource.
        Name must be unique within a namespace. It's required when creating
        resources, although some resources may allow a client to request the
        generation of an appropriate name automatically.
        Name is primarily intended for creation idempotence and configuration
        definition. Cannot be updated. More info:
        http://kubernetes.io/docs/user-guide/identifiers#names
    labels: {str:str}, optional
        Labels is a map of string keys and values that can be used to
        organize and categorize (scope and select) objects.
        May match selectors of replication controllers and services.
        More info: http://kubernetes.io/docs/user-guide/labels
    annotations: {str:str}, optional
        Annotations is an unstructured key value map stored with a
        resource that may be set by external tools to store and retrieve
        arbitrary metadata. They are not queryable and should be preserved
        when modifying objects.
        More info: http://kubernetes.io/docs/user-guide/annotations
    namespace: str, default is default, optional
        Namespaces are intended for use in environments with many users spread
        across multiple teams, or projects.
        For clusters with a few to tens of users, you should not need to create
        or think about namespaces at all. Start using namespaces when you need the features they provide.
        More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/
    '''
    name: str
    labels?: {str: str}
    annotations?: {str: str}
    namespace?: str

schema AppConfiguration:
    '''    AppConfiguration is the common user interface for long-running
    services adopting the best practice of Kubernetes.

    Attributes
    ----------
    workloadType: str, default is Deployment
        Use this attribute to specify which kind of long-running service you want.
        Valid values: Deployment, CafeDeployment.
        Default to Deployment.
        See also: kusion_models/core/v1/workload_metadata.k
    name: str
        Required.
        A Server-level attribute.
        The name of the long-running service.
        See also: kusion_models/core/v1/metadata.k
    namespace: str, default = default
        Required.
        A Server-level attribute.
        The namespace of the long-running service.
        See also: kusion_models/core/v1/metadata.k
    app: str
        A Server-level attribute.
        The name of the application.
        If specified, it will be used as the value of the default label "app".
        If not specified, the value of the attribute name will be used.
        See also: kusion_models/core/v1/workload_metadata.k

    Examples
    --------
    myCustomApp = AppConfiguration {
        name: "componentName"
    }
    '''
    workloadType: str = "Deployment"
    name: str
    namespace: str = "default"
    app: str