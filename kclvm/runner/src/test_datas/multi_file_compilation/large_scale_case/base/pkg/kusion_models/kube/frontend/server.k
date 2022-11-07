import base.pkg.kusion_kubernetes.apimachinery.apis
import base.pkg.kusion_models.kube.frontend.configmap
import base.pkg.kusion_models.kube.frontend.secret
import base.pkg.kusion_models.kube.frontend.service
import base.pkg.kusion_models.kube.frontend.serviceaccount as sa
import base.pkg.kusion_models.kube.frontend.ingress
import base.pkg.kusion_models.kube.frontend.container
import base.pkg.kusion_models.kube.frontend.volume
import base.pkg.kusion_models.kube.frontend.sidecar as s
import base.pkg.kusion_models.kube.frontend.strategy

schema Server:
    """Server is abstaction of Deployment and StatefulSet.

    Attributes
    ----------
    workloadType: "Deployment" | "StatefulSet", default is "Deployment", required.
        Application workload type, default to 'Deployment'
    replicas: int, default is 1, required.
        Number of desired pods. This is a pointer to distinguish between explicit zero and not specified. Defaults to 1.
    image: str, default is Undefined, required.
        Container image name.
        More info: https://kubernetes.io/docs/concepts/containers/images
    schedulingStrategy: strategy.SchedulingStrategy, default is Undefined, required.
        SchedulingStrategy represents scheduling strategy.
    mainContainer: container.Main, default is Undefined, required.
        MainContainer describes the main container configuration that is expected to be run on the host.
    sidecarContainers: [s.Sidecar], default is Undefined, optional.
        SidecarContainers describes the list of sidecar container configuration that is expected to be run on the host.
    initContainers: [s.Sidecar], default is Undefined, optional.
        InitContainers describes the list of sidecar container configuration that is expected to be run on the host.
    useBuiltInLabels: bool, default is False, optional.
        UseBuiltInLabels indicates use built-in labels or not.
    labels: {str:str}, default is Undefined, optional.
        Labels is a map of string keys and values that can be used to organize and categorize (scope and select) objects.
        More info: http://kubernetes.io/docs/user-guide/labels
    annotations: {str:str}, default is Undefined, optional
        Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata.
        More info: http://kubernetes.io/docs/user-guide/annotations
    useBuiltInSelector: bool, default is False, optional.
        UseBuiltInSelector indicates use built-in selector or not.
    selector: {str:str}, default is Undefined, optional.
        Label selector for pods. Existing ReplicaSets/ whose pods are selected by this will be the ones affected by this deployment.
    podMetadata: apis.ObjectMeta, default is Undefined, optional.
        PodMetadata is metadata that all persisted resources must have, which includes all objects users must create.
    volumes: [volume.Volume], default is Undefined, optional.
        Volumes represents a named volume and corresponding mounts in containers.
    needNamespace: bool, default is True, optional.
        NeedNamespace mark server is namespace scoped or not.
    enableMonitoring: bool, default is False, optional.
        EnableMonitoring mark server is enable monitor or not.
    configMaps: [configmap.ConfigMap], default is Undefined, optional.
        ConfigMaps is a list of ConfigMap which holds configuration data for server to consume.
    secrets: [secret.Secret], default is Undefined, optional.
        Secrets is a list of Secret which hold secret data of a certain type. 
    services: [service.Service], default is Undefined, optional.
        Services is a list of Service which partition a single Kubernetes cluster into multiple virtual clusters.
    ingresses: [ingress.Ingress], default is Undefined, optional.
        Ingresses is a list of Ingress which is collection of rules that allow inbound connections to reach the endpoints defined by a backend.
    serviceAccount: sa.ServiceAccount, default is Undefined, optional.
        ServiceAccount is used to run this pod.

    Examples
    --------
    import base.pkg.kusion_models.kube.frontend
    import base.pkg.kusion_models.kube.frontend.container
    import base.pkg.kusion_models.kube.templates.resource as res_tpl

    appConfiguration: frontend.Server {
        mainContainer = container.Main {
            name = "php-redis"
            env = [
                {
                    name = "GET_HOSTS_FROM"
                    value = "dns"
                }
            ]
            ports = [{containerPort = 80}]
        }
        selector = {
            tier = "frontend"
        }
        podMetadata.labels: {
            tier = "frontend"
        }
        schedulingStrategy.resource = res_tpl.tiny
    }
    """
    __settings__: {str:str} = {"output_type" = "IGNORE"}

    # Application workload type, default to 'Deployment'
    workloadType: "Deployment" | "StatefulSet" = "Deployment"

    # Application replicas
    replicas: int = option("replicas") or 1

    # Main container image
    image: str = option("image")
    # Main container resource
    schedulingStrategy: strategy.SchedulingStrategy = strategy.SchedulingStrategy {}

    # Main container configuration
    mainContainer: container.Main
    # Sidecar container configurations
    sidecarContainers?: [s.Sidecar]
    # Init container configurations
    initContainers?: [s.Sidecar]

    # Workload configuration
    useBuiltInLabels?: bool = True
    labels?: {str:str}
    annotations?: {str:str}
    useBuiltInSelector?: bool = True
    selector?: {str:str}
    podMetadata?: apis.ObjectMeta
    volumes?: [volume.Volume]

    # Other configurations
    needNamespace?: bool = True
    enableMonitoring?: bool = False

    configMaps?: [configmap.ConfigMap]
    secrets?: [secret.Secret]
    services?: [service.Service]
    ingresses?: [ingress.Ingress]
    serviceAccount?: sa.ServiceAccount
