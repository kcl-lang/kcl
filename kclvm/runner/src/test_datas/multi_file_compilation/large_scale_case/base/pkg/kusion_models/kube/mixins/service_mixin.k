import base.pkg.kusion_kubernetes.api.core.v1
import base.pkg.kusion_models.kube.metadata
import base.pkg.kusion_models.kube.utils
import base.pkg.kusion_models.kube.resource
import base.pkg.kusion_models.kube.protocol

mixin ServiceMixin for protocol.ServerProtocol:
    if config.services:
        assert "workloadType" in config, "config must contain `workloadType` attribute"

        kubernetes: resource.ResourceMapping {
            Service = [v1.Service {
                metadata: utils.MetadataBuilder(_s) | {
                    name = _s?.name or "{}{}{}".format(metadata.__META_APP_NAME, metadata.__META_ENV_TYPE_NAME, _i)
                }
                spec: {
                    clusterIP = _s?.clusterIP if _s?.type in ["ClusterIP", "NodePort", "LoadBalancer", None, Undefined] else Undefined
                    externalIPs = _s?.externalIPs
                    externalName = _s?.externalName if _s?.type == "ExternalName" else Undefined
                    externalTrafficPolicy = _s?.externalTrafficPolicy
                    healthCheckNodePort = _s?.healthCheckNodePort if _s?.type == "LoadBalancer" and _s?.externalTrafficPolicy == "Local" else Undefined
                    ipFamilyPolicy = _s?.ipFamilyPolicy
                    loadBalancerIP = _s?.loadBalancerIP if _s?.type == "LoadBalancer" else Undefined
                    loadBalancerSourceRanges = _s?.loadBalancerSourceRanges
                    ports = _s?.ports
                    publishNotReadyAddresses = _s?.publishNotReadyAddresses
                    selector = _s?.selector or kubernetes?[config.workloadType]?[0]?.spec?.selector?.matchLabels
                    sessionAffinity = _s?.sessionAffinity
                    sessionAffinityConfig = _s?.sessionAffinityConfig if _s?.sessionAffinity == "ClientIP" else Undefined
                    type = _s?.type
                }
            } for _i, _s in config.services]
        }
