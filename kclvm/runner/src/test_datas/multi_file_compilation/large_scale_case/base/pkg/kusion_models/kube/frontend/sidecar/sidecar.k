import base.pkg.kusion_models.kube.frontend.container.probe as p
import base.pkg.kusion_models.kube.frontend.container.env as e
import base.pkg.kusion_models.kube.frontend.container.port as cp
import base.pkg.kusion_models.kube.frontend.container.lifecycle as lc
import base.pkg.kusion_models.kube.frontend.resource as res

schema Sidecar:
    """ Sidecar describes the sidecar container configuration that is expected to be run on the host.

    Attributes
    ----------
    name: str, default is Undefined, required.
        A Container-level attribute.
        The container name. Each container in a pod must have a unique name.
    command: [str], default is Undefined, optional.
        A Container-level attribute.
        The startup command of main process. The image's entrypoint is used if this is not provided.
    args: [str], default is Undefined, optional.
        A Container-level attribute.
        The startup arguments of main process. The image's cmd is used if this is not provided.
    env: [e.Env], default is Undefined, optional.
        A Container-level attribute.
        List of environment variables in the container.
    envFrom: [e.EnvFromSource], default is Undefined, optional
        A Container-level attribute.
        List of sources to populate environment variables in the container.
    image: str, default is Undefined, required
        A Container-level attribute.
        Container image name. More info: https://kubernetes.io/docs/concepts/containers/images
    livenessProbe: p.Probe, default is Undefined, optional.
        A Container-level attribute.
        The probe to check whether container is live or not.
    readinessProbe: p.Probe, default is Undefined, optional.
        A Container-level attribute.
        The probe to check whether container is ready or not.
    startupProbe: p.Probe, default is Undefined, optional.
        A Container-level attribute.
        The probe to indicates that the Pod has successfully initialized.
    resource: str | res.Resource, default is "1<cpu<2,1Gi<memory<2Gi,disk=20Gi", required.
        A Pod-level attribute.
        Sidecar container resource. 
    lifecycle: lc.Lifecycle, default is Undefined, optional
        Actions that the management system should take in response to container lifecycle events.
        Cannot be updated.
    workingDir: str, default is Undefined, optional
        Container's working directory. If not specified, the container runtime's default will be used, 
        which might be configured in the container image. Cannot be updated.
    securityContext: {str:}, default is Undefined, optional
        SecurityContext defines the security options the container should be run with.
        If set, the fields of SecurityContext override the equivalent fields of PodSecurityContext.
        More info: https://kubernetes.io/docs/tasks/configure-pod-container/security-context/
    ports: [cp.ContainerPort], default is Undefined, optional
        List of ports to expose from the container. Exposing a port here gives the system additional information about the network connections a container uses, but is primarily informational. Not specifying a port here DOES NOT prevent that port from being exposed. Any port which is listening on the default "0.0.0.0" address inside a container will be accessible from the network. Cannot be updated.

    Examples
    --------
    import base.pkg.kusion_models.kube.frontend.sidecar as s
    import base.pkg.kusion_models.kube.frontend.container.probe as p

    sidecar = s.Sidecar {
        name = "test"
        livenessProbe = p.Probe {
            handler = p.Http {
                httpPath = "/healthz"
            }
            initialDelaySeconds = 10
        }
    }
    """

    name: str
    command?: [str]
    args?: [str]
    env?: [e.Env]
    envFrom?: [e.EnvFromSource]
    ports?: [cp.ContainerPort]
    resource: str | res.Resource
    image: str

    readinessProbe?: p.Probe
    livenessProbe?: p.Probe
    startupProbe?: p.Probe

    lifecycle?: lc.Lifecycle
    workingDir?: str
    securityContext?: {str:}

    check:
        livenessProbe.successThreshold == 1 if livenessProbe, "success threshold must be 1 for liveness probe"
