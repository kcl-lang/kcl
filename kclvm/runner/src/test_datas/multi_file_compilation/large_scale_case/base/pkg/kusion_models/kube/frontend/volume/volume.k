import regex

schema Volume:
    """Volume represents a named volume and corresponding mounts in containers.

    Attributes
    ----------
    name: str, default is Undefined, required.
        Volume's name. Must be a DNS_LABEL and unique within the pod. 
        More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#names
    volumeSource: EmptyDir | Secret | ConfigMap | FlexVolume | HostPath | DownwardAPI | CSI, default is Undefined, required.
        VolumeSource represents the location and type of the mounted volume.
    mounts: [Mount], default is Undefined, optional.
        Volumes to mount into the container's filesystem.

    Examples
    --------
    volume = v.Volume {
        name = "kubeconfig"
        volumeSource = v.Secret {
            secretName = "kubeconfig"
            defaultMode =  420
        }
        mounts = [
            v.Mount {
                path = "/etc/kubernetes/kubeconfig"
                readOnly = true
            }
        ]
    }
    """

    name: str
    volumeSource: EmptyDir | Secret | ConfigMap | FlexVolume | HostPath | DownwardAPI | CSI
    mounts?: [Mount]

schema Mount:
    """ Mount represents a mounting of a Volume within a container.

    Attributes
    ----------
    container: str, default is *, required.
        A Pod-level attribute.
        Name of container to mount, * represents all containers.
    path: str, default is Undefined, required.
        A Container-level attribute.
        Path within the container at which the volume should be mounted.
    subPath: str, default is Undefined, optional.
        A Container-level attribute.
        Path within the volume from which the container's volume should be mounted.
    readOnly: bool, default is False, optional.
        A Container-level attribute.
        Mounted read-only if true, read-write otherwise.
    """

    container: str = "*"
    path: str
    subPath?: str
    readOnly?: bool = False

    check:
        ":" not in path, "path must not contain ':'"

schema EmptyDir:
    """ EmptyDir represents a temporary directory that shares a pod's lifetime.

    Attributes
    ----------
    medium: "" | "Memory", default is "", required.
        A Pod-level attribute.
        What type of storage medium should back this directory.
    sizeLimit: str, default is Undefined, optional.
        A Pod-level attribute.
        Total amount of local storage required for this EmptyDir volume.
    """

    medium: "" | "Memory" = ""
    sizeLimit?: str

    check:
        regex.match(sizeLimit, r"^([1-9][0-9]{0,63})(E|P|T|G|M|K|Ei|Pi|Ti|Gi|Mi|Ki)$") if sizeLimit, "a valid size must match the target expression"

schema Secret:
    """ Secret represents a secret that should populate this volume.

    Attributes
    ----------
    secretName: str, default is Undefined, required.
        A Pod-level attribute.
        Name of the secret in the pod's namespace to use.
    items: [{str:str}], default is Undefined, optional.
        A Pod-level attribute.
        Key-value pairs projected into the volume.
    defaultMode: int, default is Undefined, optional.
        A Pod-level attribute.
        Mode bits used to set permissions on created files by default.
    """

    secretName: str
    items?: [{str:str}]
    defaultMode?: int

schema ConfigMap:
    """ ConfigMap represents a secret that should populate this volume.

    Attributes
    ----------
    name: str, default is Undefined, required.
        A Pod-level attribute.
        Name of the configMap in the pod's namespace to use.
    items: [{str:str}], default is Undefined, optional.
        A Pod-level attribute.
        Key-value pairs projected into the volume.
    defaultMode: int, default is Undefined, optional.
        A Pod-level attribute.
        Mode bits used to set permissions on created files by default.
    """

    name: str
    items?: [{str:str}]
    defaultMode?: int

schema FlexVolume:
    """ FlexVolume represents a secret that should populate this volume.

    Attributes
    ----------
    driver: str, default is Undefined, required.
        A Pod-level attribute.
        Driver is the name of the driver to use for this volume.
    fsType: str, default is Undefined, optional.
        A Pod-level attribute.
        Filesystem type to mount. Must be a filesystem type supported by the host operating system. Ex. "ext4", "xfs", "ntfs". 
        The default filesystem depends on FlexVolume script.
    options: {str:str}, default is Undefined, optional.
        A Pod-level attribute.
        Extra command options if any.
    readOnly: bool, default is False, optional.
        A Pod-level attribute.
        Optional: Defaults to false (read/write). ReadOnly here will force the ReadOnly setting in VolumeMounts.
    """

    driver: str
    fsType?: str
    options?: {str:str}
    readOnly?: bool

schema HostPath:
    """ HostPath represents a secret that should populate this volume.

    Attributes
    ----------
    path: str, default is Undefined, required.
        A Pod-level attribute.
        Path of the directory on the host. If the path is a symlink, it will follow the link to the real path. 
        More info: https://kubernetes.io/docs/concepts/storage/volumes#hostpath
    type: str, default is Undefined, optional.
        A Pod-level attribute.
        Type for HostPath Volume Defaults to "" 
        More info: https://kubernetes.io/docs/concepts/storage/volumes#hostpath
    """

    path: str
    type?: str

schema DownwardAPI:
    """ DownwardAPI represents a secret that should populate this volume.

    Attributes
    ----------
    defaultMode: int, default is Undefined, optional.
        A Pod-level attribute.
        Mode bits used to set permissions on created files by default.
    items: [{str:}], default is Undefined, optional.
        A Pod-level attribute.
        Items is a list of downward API volume file
    """

    defaultMode?: int
    items?: [{str:}]

schema CSI:
    """ CSI (Container Storage Interface) represents ephemeral storage that is handled by certain external CSI drivers (Beta feature).

    Attributes
    ----------
    driver: str, default is Undefined, required.
        A Pod-level attribute.
        Driver is the name of the driver to use for this volume.
    fsType: str, default is Undefined, optional.
        A Pod-level attribute.
        Filesystem type to mount. Must be a filesystem type supported by the host operating system. Ex. "ext4", "xfs", "ntfs". 
        The default filesystem depends on FlexVolume script.
    readOnly: bool, default is False, optional.
        A Pod-level attribute.
        Optional: Defaults to false (read/write). ReadOnly here will force the ReadOnly setting in VolumeMounts.
    volumeAttributes: {str:str}, default is Undefined, optional.
        A Pod-level attribute.
        Extra command options if any.
    """
    driver: str
    fsType?: str
    readOnly?: bool
    volumeAttributes?: {str:str}
