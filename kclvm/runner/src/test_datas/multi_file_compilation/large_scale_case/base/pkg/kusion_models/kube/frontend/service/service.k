import base.pkg.kusion_models.kube.frontend.common
import base.pkg.kusion_kubernetes.api.core.v1 as corev1

schema Service(common.Metadata):
    """Service are Kubernetes objects which partition a single Kubernetes cluster into multiple virtual clusters.
    More info: https://kubernetes.io/docs/reference/kubernetes-api/service-resources/service-v1/#Service

    Attributes
    ----------
    ports: [{str:}], default is Undefined, optional.
        The list of ports that are exposed by this service. 
        More info: https://kubernetes.io/docs/concepts/services-networking/service/#virtual-ips-and-service-proxies
    selector: {str:str}, default is Undefined, optional.
        Route service traffic to pods with label keys and values matching this selector.
        More info: https://kubernetes.io/docs/concepts/services-networking/service/
    type: str, default is "ClusterIP", optional.
        determines how the Service is exposed.
        More info: https://kubernetes.io/docs/concepts/services-networking/service/#publishing-services-service-types
    clusterIP: str, default is None, optional.
        clusterIP is the IP address of the service and is usually assigned randomly by the master.
        More info: https://kubernetes.io/docs/concepts/services-networking/service/#virtual-ips-and-service-proxies
    externalIPs: [str], default is Undefined, optional
        externalIPs is a list of IP addresses for which nodes in the cluster will also accept traffic for this service.
    externalName: str, default is Undefined, optional
        externalName is the external reference that discovery mechanisms will return as an alias for this service (e.g. a DNS CNAME record).
    externalTrafficPolicy: str, default is Undefined, optional
        externalTrafficPolicy denotes if this Service desires to route external traffic to node-local or cluster-wide endpoints.
    healthCheckNodePort: int, default is Undefined, optional
        healthCheckNodePort specifies the healthcheck nodePort for the service.
    ipFamilyPolicy: str, default is Undefined, optional
        ipFamilyPolicy represents the dual-stack-ness requested or required by this Service, and is gated by the "IPv6DualStack" feature gate.
    loadBalancerIP: str, default is Undefined, optional
        Only applies to Service Type: LoadBalancer LoadBalancer will get created with the IP specified in this field.
    loadBalancerSourceRanges: [str], default is Undefined, optional
        If specified and supported by the platform, this will restrict traffic through the cloud-provider load-balancer will be restricted to the specified client IPs.
        This field will be ignored if the cloud-provider does not support the feature.
        More info: https://kubernetes.io/docs/tasks/access-application-cluster/create-external-load-balancer/
    ports: [corev1.ServicePort], default is Undefined, optional
        The list of ports that are exposed by this service. 
        More info: https://kubernetes.io/docs/concepts/services-networking/service/#virtual-ips-and-service-proxies
    publishNotReadyAddresses: bool, default is Undefined, optional
        publishNotReadyAddresses indicates that any agent which deals with endpoints for this Service should disregard any indications of ready/not-ready.
    sessionAffinity: str, default is Undefined, optional
        Supports "ClientIP" and "None". Used to maintain session affinity.
        More info: https://kubernetes.io/docs/concepts/services-networking/service/#virtual-ips-and-service-proxies
    sessionAffinityConfig: {str:}, default is Undefined, optional
        sessionAffinityConfig contains the configurations of session affinity.

    Examples
    --------
    service = Service {
        name = "my-service-name"
        namespace = "my-service-name"
        labels.env = "dev"
        ports = [
            {
                name = "grpc-xds"
                port = 15010
            }
            {
                name = "https-xds"
                port = 15011
            }
        ]
    }
    """
    selector?: {str:str}
    ports?: [corev1.ServicePort]
    clusterIP?: str
    type?: str
    externalIPs?: [str]
    externalName?: str
    externalTrafficPolicy?: str
    healthCheckNodePort?: int
    ipFamilyPolicy?: str
    loadBalancerIP?: str
    loadBalancerSourceRanges?: [str]
    publishNotReadyAddresses?: bool
    sessionAffinity?: str
    sessionAffinityConfig?: {str:}
