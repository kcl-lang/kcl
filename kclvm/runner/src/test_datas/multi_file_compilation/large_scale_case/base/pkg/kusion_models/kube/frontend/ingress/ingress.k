import base.pkg.kusion_models.kube.frontend.common
import base.pkg.kusion_kubernetes.api.networking.v1 as networkingv1

schema Ingress(common.Metadata):
    """Ingress is a collection of rules that allow inbound connections to reach the endpoints defined by a backend. 
    An Ingress can be configured to give services externally-reachable urls, load balance traffic, terminate SSL, offer name based virtual hosting etc.

    Attributes
    ----------
    rules: [networkingv1.IngressRule], default is Undefined, optional
        A list of host rules used to configure the Ingress. If unspecified, or no rule matches, all traffic is sent to the default backend.
    tls: [networkingv1.IngressTLS], default is Undefined, optional
        TLS configuration. Currently the Ingress only supports a single TLS port, 443. If multiple members of this list specify different hosts, they will be multiplexed on the same port according to the hostname specified through the SNI TLS extension, if the ingress controller fulfilling the ingress supports SNI.

    Examples
    --------
    ingress.Ingress {
        name = "example-ingress"
        rules = [
            {
                host = "your-domain.com"
                http.paths = [
                    {
                        path = "/apple"
                        pathType = "Prefix"
                        backend.service: {
                            name = "app-service"
                            port.number = 5678
                        }
                    }
                ]
            }
        ]
        tls = [
            {
                hosts = ["your-domain.com"]
                secretName = "example-ingress-tls"
            }
        ]
    }
    """
    rules?: [networkingv1.IngressRule]
    tls?: [networkingv1.IngressTLS]
