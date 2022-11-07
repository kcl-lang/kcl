import base.pkg.kusion_models.kube.frontend.container.probe as p
import base.pkg.kusion_models.kube.frontend.container.env as e
import base.pkg.kusion_models.kube.frontend.container.port as cp
import base.pkg.kusion_models.kube.frontend.container.lifecycle as lc

schema Main:
    """ Main describes the main container configuration that is expected to be run on the host.

    Attributes
    ----------
    name: str, default is "main", required.
        A Container-level attribute.
        The container name. Each container in a pod must have a unique name.
    command: [str], default is Undefined, optional.
        A Container-level attribute.
        The startup command of main process. The image's entrypoint is used if this is not provided.
    useBuiltInEnv: bool, default is False, optional.
        useBuiltInEnv indicates use built-in envs or not.
    args: [str], default is Undefined, optional.
        A Container-level attribute.
        The startup arguments of main process. The image's cmd is used if this is not provided.
    env: [e.Env], default is Undefined, optional.
        A Container-level attribute.
        List of environment variables in the container.
    envFrom: [e.EnvFromSource], default is Undefined, optional.
        A Container-level attribute.
        List of sources to populate environment variables in the container.
    ports: [cp.ContainerPort], default is Undefined, optional.
        A Container-level attribute.
        List of network ports in the container.
    livenessProbe: p.Probe, default is Undefined, optional.
        A Container-level attribute.
        The probe to check whether container is live or not.
    readinessProbe: p.Probe, default is Undefined, optional.
        A Container-level attribute.
        The probe to check whether container is ready or not.
        The default value can be referred to presupposed template: base/pkg/kusion_models/templates/sofa_probe.k
    startupProbe: p.Probe, default is Undefined, optional.
        A Container-level attribute.
        The probe to indicates that the Pod has successfully initialized.
    lifecycle: lc.Lifecycle, default is Undefined, optional
        Actions that the management system should take in response to container lifecycle events. Cannot be updated.
    securityContext: {str:any}, default is Undefined, optional
        SecurityContext defines the security options the container should be run with. If set, the fields of SecurityContext override the equivalent fields of PodSecurityContext. More info: https://kubernetes.io/docs/tasks/configure-pod-container/security-context/
    workingDir: str, default is Undefined, optional
        Container's working directory. If not specified, the container runtime's default will be used, which might be configured in the container image. Cannot be updated.

    Examples
    --------
    import base.pkg.kusion_models.kube.frontend.container
    import base.pkg.kusion_models.kube.frontend.container.probe as p
    
    main = container.Main {
        name = "test"
        livenessProbe = p.Probe {
            handler = p.Http {
                path = "/healthz"
            }
            initialDelaySeconds = 10
        }
    }
    """

    name: str = "main"
    command?: [str]
    args?: [str]
    useBuiltInEnv?: bool = False
    env?: [e.Env]
    envFrom?: [e.EnvFromSource]
    ports?: [cp.ContainerPort]

    livenessProbe?: p.Probe
    readinessProbe?: p.Probe
    startupProbe?: p.Probe

    lifecycle?: lc.Lifecycle
    workingDir?: str
    securityContext?: {str:}

    check:
        livenessProbe.successThreshold == 1 if livenessProbe, "success threshold must be 1 for liveness probe"
