import base.pkg.kusion_kubernetes.api.core.v1
import base.pkg.kusion_models.kube.metadata
import base.pkg.kusion_models.kube.utils
import base.pkg.kusion_models.kube.resource
import base.pkg.kusion_models.kube.protocol

mixin ServiceAccountMixin for protocol.ServerProtocol:
    if config.serviceAccount:
        kubernetes: resource.ResourceMapping {
            ServiceAccount = [v1.ServiceAccount {
                metadata: utils.MetadataBuilder(config.serviceAccount) | {
                    name = config.serviceAccount.name
                }
                imagePullSecrets = config.serviceAccount.imagePullSecrets
                secrets = config.serviceAccount.secrets
            }]
        }
