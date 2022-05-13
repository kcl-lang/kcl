import config_map.core.metadata

schema ConfigMap(metadata.Metadata):
    # 目的是根据入参生成 Kubernetes 资源
    # 入参列表
    data?: {str:str}
    binaryData?: {str:str}
