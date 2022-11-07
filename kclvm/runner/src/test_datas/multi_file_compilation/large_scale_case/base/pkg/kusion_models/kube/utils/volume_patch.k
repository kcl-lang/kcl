import base.pkg.kusion_models.kube.frontend.volume

VolumePatch = lambda volumes: [volume.Volume], containers: [{str:}] -> [{str:}] {
    """VolumePatch patches volumes into _containers and returns the patched _container"""
    [
        _container | {
            volumeMounts = sum([[
                {
                    name: volume.name
                    mountPath: mount.path
                    subPath: mount.subPath
                    readOnly: mount.readOnly or Undefined
                } for mount in volume.mounts if mount.container == _container.name or mount.container == "*"
            ] for volume in volumes] if volumes else [], [])
        } for _container in containers
    ] if containers else Undefined
}
