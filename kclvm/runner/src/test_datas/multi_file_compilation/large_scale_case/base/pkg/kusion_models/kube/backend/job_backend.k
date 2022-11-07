import base.pkg.kusion_kubernetes.api.batch.v1 as batchv1
import base.pkg.kusion_models.kube.frontend
import base.pkg.kusion_models.kube.mixins
import base.pkg.kusion_models.kube.utils
import base.pkg.kusion_models.kube.metadata
import base.pkg.kusion_models.kube.resource

schema JobBackend[inputConfig: frontend.Job]:
    # mixins
    mixin [
        # Resource builder mixin
        mixins.NamespaceMixin,
        mixins.ConfigMapMixin,
        mixins.ServiceAccountMixin

        # mixins.OutputTypeMixin
    ]

    config: frontend.Job = inputConfig

    # variables
    jobName: str = "{}-{}".format(metadata.__META_APP_NAME, metadata.__META_ENV_TYPE_NAME).lower()
    app: utils.ApplicationBuilder = utils.ApplicationBuilder {}
    mainContainerDict: {str:}
    mainContainer: {str:}
    sidecarContainers?: [{str:}]
    initContainers?: [{str:}]

    # rendering logic
    if config.mainContainer:
        assert config.image, "config.image must be specified and can't be empty or None or Undefined"
        # construct input for converter
        mainContainerDict = {
            **config.mainContainer
            if config.mainContainer.useBuiltInEnv:
                env += app.envs
            name = config.mainContainer.name or "main"
            image = config.image
            resource = config?.schedulingStrategy?.resource
        }
        mainContainer = utils.VolumePatch(config.volumes, [utils.ContainerFrontend2Kube(mainContainerDict)])?[0]

    if config.sidecarContainers:
        sidecarContainers = utils.VolumePatch(config.volumes, [utils.ContainerFrontend2Kube(_s) for _s in config.sidecarContainers])

    if config.initContainers:
        initContainers =  utils.VolumePatch(config.volumes, [utils.ContainerFrontend2Kube(_s) for _s in config.initContainers])

    # construct job attributes
    jobAttrs: {str:} = {
        metadata = utils.MetadataBuilder(config) | {
            name = jobName
        }
        spec = {
            activeDeadlineSeconds = config.activeDeadlineSeconds
            backoffLimit = config.backoffLimit
            completionMode = config.completionMode
            completions = config.completions
            manualSelector = config.manualSelector
            parallelism = config.parallelism
            suspend = config.suspend
            ttlSecondsAfterFinished = config.ttlSecondsAfterFinished
            selector.matchLabels = app.selector | config.selector

            template = {
                metadata = {
                    labels = app.labels
                    **config.podMetadata
                }
                spec = {
                    containers = [
                        mainContainer
                        *sidecarContainers
                    ]
                    initContainers = initContainers
                    restartPolicy = config.restartPolicy
                    # volume
                    if config.volumes: volumes = [
                        (lambda volume {
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
                    # service account
                    if config.serviceAccount:
                        serviceAccountName = config.serviceAccount.name
                }
            }
        }
    }

    # generate job instance
    _jobInstance = batchv1.Job {**jobAttrs}

    # put job instance to backend result
    kubernetes: resource.ResourceMapping = {
        "${typeof(_jobInstance)}" = [_jobInstance]
    }
