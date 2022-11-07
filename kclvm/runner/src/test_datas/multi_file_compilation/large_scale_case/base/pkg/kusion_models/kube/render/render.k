import base.pkg.kusion_models.kube.frontend
import base.pkg.kusion_models.kube.frontend.rbac
import base.pkg.kusion_models.kube.backend
import base.pkg.kusion_models.kube.resource
import base.pkg.kusion_kubernetes.api.core.v1 as corev1
import base.pkg.kusion_prometheus.monitoring.v1 as monitoringv1
import base.pkg.kusion_prometheus.monitoring.v1alpha1 as monitoringv1alpha1

# prededined constants
__output_standalone__ = {__settings__: {output_type = "STANDALONE"}}
__output_ignore__ = {__settings__: {output_type = "IGNORE"}}
__output_inline__ = {__settings__: {output_type = "INLINE"}}

__renderServerFrontendInstances__ = [inst for inst in frontend.Server.instances() if typeof(inst) == "Server"]
__renderServerBackendInstances__ = [backend.ServerBackend(inst) for inst in __renderServerFrontendInstances__]

__renderJobFrontendInstances__ = [inst for inst in frontend.Job.instances() if typeof(inst) == "Job"]
__renderJobBackendInstances__ = [backend.JobBackend(inst) for inst in __renderJobFrontendInstances__]
__renderFrontendInstances__ = [
    *__renderServerFrontendInstances__
    *__renderJobFrontendInstances__
]
__renderBackendInstances__ = [
    *__renderServerBackendInstances__
    *__renderJobBackendInstances__
]

# rbac map, kind to resource array
__rbac_map__: {str:[]} = {
    if len(rbac.ClusterRoleBinding.instances()) > 0:
        ClusterRoleBinding += [r.kubernetes for r in rbac.ClusterRoleBinding.instances()]
    if len(rbac.ClusterRole.instances()) > 0: 
        ClusterRole += [r.kubernetes for r in rbac.ClusterRole.instances()]
    if len(rbac.RoleBinding.instances()) > 0: 
        RoleBinding += [r.kubernetes for r in rbac.RoleBinding.instances()]
    if len(rbac.Role.instances()) > 0: 
        Role += [r.kubernetes for r in rbac.Role.instances()]
}

# prometheus
__prometheus_map__: {str:[]} = {
    PrometheusRule += monitoringv1.PrometheusRule.instances()
    AlertmanagerConfig += monitoringv1alpha1.AlertmanagerConfig.instances()
    Probe += monitoringv1.Probe.instances()
    Prometheus += monitoringv1.Prometheus.instances()
    Alertmanager += monitoringv1.Alertmanager.instances()
}

# k8s
__k8s__: {str:[]} = {
    Service += corev1.Service.instances()
    ServiceAccount += corev1.ServiceAccount.instances()
}

# merge resource map
__array_of_resource_map___: [{str:[]}] = [
    _inst?.kubernetes | __output_standalone__ \
    for _inst in __renderBackendInstances__ if _inst?.kubernetes
] | [__rbac_map__] | [__prometheus_map__] | [__k8s__]

# remove duplicates
__resource_map___: {str: []} = {
    _kind: [r for key, r in _res] \
        for _index, _resource_map in __array_of_resource_map___  \
            for _kind, _res in _resource_map if _kind not in ["__settings__"]
}

# structure output
if option("__konfig_output_format__")?.lower() == "raw":
    konfig_kubeKubernetes = { 
        _kind: [_resource | __output_standalone__ for _resource in _res] \
            for _kind, _res in __resource_map___
    } | __output_ignore__
else:
    # kubernetes id: apiVersion:Kind:Namespace:Name
    getId = lambda _resource {
        _resource.apiVersion + ":" + _resource.kind + ":" + (_resource.metadata.namespace + ":" if _resource.metadata.namespace else "") + _resource.metadata.name 
    }
    x = {
        _kind: [
            {
                "id" = getId(_resource)
                "type" = "Kubernetes"
                "dependsOn" = [
                    getId(dependOn) \
                        for i, v in resource.kubernetesOrders if v == _kind \
                            for k in resource.kubernetesOrders[:i] if k in __resource_map___ \
                                for dependOn in __resource_map___[k]
                ]
                "attributes" = _resource | __output_inline__
            } | __output_standalone__ for _resource in _res
        ] for _kind, _res in __resource_map___ 
    } | __output_ignore__
