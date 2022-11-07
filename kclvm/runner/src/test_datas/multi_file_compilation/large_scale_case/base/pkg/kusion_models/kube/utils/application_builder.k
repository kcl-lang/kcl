import base.pkg.kusion_models.kube.metadata

schema ApplicationBuilder:
    """ApplicationBuilder contains the workload labels, selector and environments about the application
    
    Reference from:  https://kubernetes.io/docs/concepts/overview/working-with-objects/common-labels/
    """
    labels: {str:str} = {
        "app.kubernetes.io/name": metadata.__META_APP_NAME
        "app.kubernetes.io/env": metadata.__META_ENV_TYPE_NAME
        "app.kubernetes.io/instance": "{}-{}".format(metadata.__META_APP_NAME, metadata.__META_ENV_TYPE_NAME).lower()
        "cluster.x-k8s.io/cluster-name": metadata.__META_CLUSTER_NAME
    }

    selector: {str:str} = labels

    envs: [{str:}] = [
        {
            name: "APP_NAME"
            value: metadata.__META_APP_NAME
        }
        {
            name: "ENVIRONMENT"
            value: metadata.__META_ENV_TYPE_NAME
        }
        {
            name: "INSTANCE"
            value: "{}-{}".format(metadata.__META_APP_NAME, metadata.__META_ENV_TYPE_NAME).lower()
        }
        {
            name: "CLUSTER"
            value: metadata.__META_CLUSTER_NAME
        }
    ]
