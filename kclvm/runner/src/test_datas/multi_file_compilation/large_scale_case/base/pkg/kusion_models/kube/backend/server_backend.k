import base.pkg.kusion_kubernetes.api.apps.v1 as appsv1
import base.pkg.kusion_kubernetes.api.core.v1 as corev1
import base.pkg.kusion_models.kube.frontend.server
import base.pkg.kusion_models.kube.mixins
import base.pkg.kusion_models.kube.metadata
import base.pkg.kusion_models.kube.utils
import base.pkg.kusion_models.kube.resource
import base.pkg.kusion_prometheus.mixins as pmixins

# Frontend volume to kubernetes volume mapping.
VOLUME_SOURCE_TYPE_MAPPING = {
    EmptyDir = "emptyDir"
    Secret = "secret"
    ConfigMap = "configMap"
    DownwardAPI = "downwardAPI"
    CSI = "csi"
    HostPath = "hostPath"
}

schema ServerBackend[inputConfig: server.Server]:
    """ServerBackend converts the user-written front-end model `Server` into a 
    collection of kubernetes resources and places the resource collection into 
    the `kubernetes` attribute.
    """
    mixin [
        # Resource builder mixin
        mixins.NamespaceMixin,
        mixins.ConfigMapMixin,
        mixins.SecretMixin,
        mixins.ServiceMixin,
        mixins.IngressMixin,
        mixins.ServiceAccountMixin,

        # Monitor mixin
        pmixins.MonitorMixin
    ]
    
    # Store the input config parameter, ensure it can be seen in protocol and mixin.
    config: server.Server = inputConfig
    # Workload name.
    workloadName: str = "{}{}".format(metadata.__META_APP_NAME, metadata.__META_ENV_TYPE_NAME).lower()
    # App variable contains labels, selector and environments.
    app: utils.ApplicationBuilder = utils.ApplicationBuilder {}
    # Main containers and sidecar contrainers.
    mainContainer: {str:}
    sidecarContainers?: [{str:}]
    initContainers?: [{str:}]

    if config.mainContainer:
        assert config.image, "config.image must be specified and can't be empty or None or Undefined"
        # Construct input of converter using the volumes.
        mainContainer = utils.VolumePatch(config.volumes, [utils.ContainerFrontend2Kube({
            **config.mainContainer
            if config.mainContainer.useBuiltInEnv:
                env += app.envs
            name = config.mainContainer.name or "main"
            image = config.image
            resource = config?.schedulingStrategy?.resource
        })])?[0]

    if config.sidecarContainers:
        sidecarContainers = utils.VolumePatch(config.volumes, [utils.ContainerFrontend2Kube(_s) for _s in config.sidecarContainers])

    if config.initContainers:
        initContainers = utils.VolumePatch(config.volumes, [utils.ContainerFrontend2Kube(_s) for _s in config.initContainers])

    # Construct workload attributes.
    workloadAttributes: {str:} = {
        metadata = utils.MetadataBuilder(config) | {
            name = workloadName
        }
        spec = {
            replicas = config.replicas
            if config.useBuiltInSelector: 
                selector.matchLabels: app.selector | config.selector
            else:
                selector.matchLabels: config.selector
            template = {
                metadata = {
                    if config.useBuiltInLabels:
                        labels = app.labels
                    **config.podMetadata                
                }
                spec = {
                    containers = [mainContainer] + (sidecarContainers or [])
                    initContainers = initContainers
                    if config.volumes: volumes = [
                        (lambda volume {
                            """Convert frontend volume to kubernetes Volume."""
                            volumeType = typeof(volume.volumeSource)
                            assert volumeType in VOLUME_SOURCE_TYPE_MAPPING, "Invalid frontend volume type, please check VOLUME_SOURCE_TYPE_MAPPING"
                            kubeVolumeType = VOLUME_SOURCE_TYPE_MAPPING[volumeType]
                            {
                                name = volume.name
                                if typeof(volume.volumeSource) == "EmptyDir" and volume.volumeSource.medium == "":
                                    "${kubeVolumeType}" = {}
                                else:
                                    "${kubeVolumeType}" = volume.volumeSource
                            }
                        })(volume) for volume in config.volumes if volume.volumeSource
                    ]
                    if config.serviceAccount:
                        serviceAccountName = config.serviceAccount.name
                }
            }
        }
    }

    # Generate workload instance.
    if config.workloadType == "Deployment":
        _workloadInstance = appsv1.Deployment {**workloadAttributes}
    elif config.workloadType == "StatefulSet":
        # Generate default headless service of StatefulSet.
        _headlessServiceInstance = corev1.Service {
            metadata = workloadAttributes.metadata
            spec = {
                clusterIP = "None"
                selector = workloadAttributes.spec.selector.matchLabels
            }
        }
        _workloadInstance = appsv1.StatefulSet {
            **workloadAttributes
            spec.serviceName = _headlessServiceInstance.metadata.name
        }
    else:
        assert False, "Invalid workload name {}".format(config.workloadType)

    # Put workload instance to backend result.
    kubernetes: resource.ResourceMapping = {
        if _workloadInstance:
            "${typeof(_workloadInstance)}" = [_workloadInstance]
        if _headlessServiceInstance:
            "${typeof(_headlessServiceInstance)}" = [_headlessServiceInstance]
    }
