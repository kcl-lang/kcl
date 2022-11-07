import base.pkg.kusion_kubernetes.apimachinery.apis
import base.pkg.kusion_models.kube.frontend.configmap
import base.pkg.kusion_models.kube.frontend.container
import base.pkg.kusion_models.kube.frontend.serviceaccount as sa
import base.pkg.kusion_models.kube.frontend.sidecar as s
import base.pkg.kusion_models.kube.frontend.strategy
import base.pkg.kusion_models.kube.frontend.volume

schema Job:
    """
    Job is the common user interface for one-time jobs, which is defined by Kubernetes Job.
    Job supports reliable parallel execution of Pods.

    Attributes
    ----------
    activeDeadlineSeconds: int, default is Undefined, optional.
        Specifies the duration in seconds relative to the startTime that the job may be active
        before the system tries to terminate it; value must be positive integer
    backoffLimit: int, default is 6, optional.
        Specifies the number of retries before marking this job failed. Defaults to 6
    completionMode: "NonIndexed" | "Indexed", default is NonIndexed, optional
        CompletionMode specifies how Pod completions are tracked. It can be `NonIndexed` (default) or `Indexed`.
    completions: int, default is Undefined, optional.
        Specifies the desired number of successfully finished pods the job should be run with.
        More info: https://kubernetes.io/docs/concepts/workloads/controllers/jobs-run-to-completion/
    manualSelector: bool, default is Undefined, optional.
        manualSelector controls generation of pod labels and pod selectors.
        More info: https://kubernetes.io/docs/concepts/workloads/controllers/jobs-run-to-completion/#specifying-your-own-pod-selector
    parallelism: int, default is Undefined, optional.
        Specifies the maximum desired number of pods the job should run at any given time.
        More info: https://kubernetes.io/docs/concepts/workloads/controllers/jobs-run-to-completion/
    suspend: bool, default is Undefined, optional
        Suspend specifies whether the Job controller should create Pods or not.
    ttlSecondsAfterFinished: int, default is Undefined, optional.
        ttlSecondsAfterFinished limits the lifetime of a Job that has finished execution (either Complete or Failed).
    selector: {str:str}, default is Undefined, optional.
        A label query over pods that should match the pod count.
        More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#label-selectors
    podMetadata: apis.ObjectMeta, default is Undefined, optional.
        PodMetadata is metadata that all persisted resources must have, which includes all objects users must create.
    labels: {str:str}, default is Undefined, optional.
        Labels is a map of string keys and values that can be used to organize and categorize (scope and select) objects.
        More info: http://kubernetes.io/docs/user-guide/labels
    annotations: {str:str}, default is Undefined, optional.
        Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata.
        More info: http://kubernetes.io/docs/user-guide/annotations
    restartPolicy: "Never" | "OnFailure", default is Never, optional.
        Restart policy for all containers within the pod. One of Always, OnFailure, Never.
        Default to Always. More info: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#restart-policy
    mainContainer: container.Main, default is Undefined, required.
        MainContainer describes the main container configuration that is expected to be run on the host.
    image: str, default is Undefined, required.
        Container image name. More info: https://kubernetes.io/docs/concepts/containers/images
    schedulingStrategy: strategy.SchedulingStrategy, default is Undefined, optional.
        SchedulingStrategy represents scheduling strategy.
    sidecarContainers: [s.Sidecar], default is Undefined, optional.
        SidecarContainers describes the list of sidecar container configuration that is expected to be run on the host.
    initContainers: [s.Sidecar], default is Undefined, optional.
        InitContainers describes the list of sidecar container configuration that is expected to be run on the host.
    needNamespace: bool, default is True, optional.
        NeedNamespace mark server is namespace scoped or not.
    volumes: [volume.Volume], default is Undefined, optional.
        Volumes represents a named volume and corresponding mounts in containers.
    configMaps: [configmap.ConfigMap], default is Undefined, optional.
        ConfigMaps is a list of ConfigMap which holds configuration data for server to consume.
    serviceAccount: sa.ServiceAccount, default is Undefined, optional.
        ServiceAccount is used to run this pod.

    Examples
    --------
    import base.pkg.kusion_models.kube.frontend

    jobConfiguration: frontend.Job {
        # main container
        mainContainer = container.Main {
            name = "pi"
            command = ["perl",  "-Mbignum=bpi", "-wle", "print bpi(2000)"]
        }
        image = "perl"
    }
    """
    __settings__: {str:str} = {"output_type" = "IGNORE"}

    # subset of batchv1.JobSpec
    activeDeadlineSeconds?: int
    backoffLimit?: int = 6
    completionMode?: "NonIndexed" | "Indexed" = "NonIndexed"
    completions?: int
    manualSelector?: bool
    parallelism?: int
    suspend?: bool
    ttlSecondsAfterFinished?: int
    selector?: {str:str}

    # subset of corev1.PodTemplateSpec
    podMetadata?: apis.ObjectMeta
    labels?: {str:str}
    annotations?: {str:str}
    restartPolicy?: "Never" | "OnFailure" = "Never"

    # main container configuration
    mainContainer: container.Main
    image: str = option("image")
    schedulingStrategy?: strategy.SchedulingStrategy = strategy.SchedulingStrategy {}

    # sidecar container configurations
    sidecarContainers?: [s.Sidecar]
    # init container configurations
    initContainers?: [s.Sidecar]

    # other
    needNamespace?: bool = True
    volumes?: [volume.Volume]
    configMaps?: [configmap.ConfigMap]
    serviceAccount?: sa.ServiceAccount
