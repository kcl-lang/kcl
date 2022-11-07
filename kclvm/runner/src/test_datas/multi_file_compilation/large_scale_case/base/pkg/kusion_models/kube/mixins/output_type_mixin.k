import base.pkg.kusion_models.kube.resource


mixin OutputTypeMixin:
    kubernetes: resource.ResourceMapping
    __output_type = "STANDALONE"
    if option("__konfig_output_format__")?.lower() == "raw":
        kubernetes = {
            "${kindName}": [_resource | {__settings__: {output_type = __output_type}} \
                for _resource in kubernetes[kindName]] \
                for kindName in kubernetes if kindName not in ["__settings__"]
        } | {__settings__: {output_type = __output_type}}
    else:
        kubernetes = {
            "${kindName}" = [{
                "id" = _resource.apiVersion + ":" + _resource.kind + ":" + (_resource.metadata.namespace + ":" if _resource.metadata.namespace else "") + _resource.metadata.name 
                "type" = "Kubernetes"
                "dependsOn" = [ dependsOn.apiVersion + ":" + dependsOn.kind + ":" + (dependsOn.metadata.namespace + ":" if dependsOn.metadata.namespace else "") + dependsOn.metadata.name for i,v in resource.kubernetesOrders if v==kindName for k in resource.kubernetesOrders[:i] if k in kubernetes for dependsOn in kubernetes[k] or []]
                "attributes" = _resource | {__settings__: {output_type = "INLINE"}}
            } | {__settings__: {output_type = __output_type}} \
                for _resource in kubernetes[kindName]] \
                for kindName in kubernetes if kindName not in ["__settings__"]
        }
