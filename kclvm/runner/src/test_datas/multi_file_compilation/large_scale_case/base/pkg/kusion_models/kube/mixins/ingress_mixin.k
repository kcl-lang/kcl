import base.pkg.kusion_kubernetes.api.networking.v1 as networking_v1
import base.pkg.kusion_models.kube.metadata
import base.pkg.kusion_models.kube.utils
import base.pkg.kusion_models.kube.resource
import base.pkg.kusion_models.kube.protocol

mixin IngressMixin for protocol.ServerProtocol:
    if config.ingresses:
        kubernetes: resource.ResourceMapping {
            Ingress = [networking_v1.Ingress {
                metadata: utils.MetadataBuilder(_s) | {
                    name = _s?.name or "{}{}{}".format(metadata.__META_APP_NAME, metadata.__META_ENV_TYPE_NAME, _i)
                }
                spec.rules = _s?.rules
                spec.tls = _s?.tls
            } for _i, _s in config.ingresses]
        }
