# KCL 格式化工具

## 简介

KCL 支持通过内置的命令行工具一键格式化多个 KCL 文件文档。本文展示 KCL 编码风格和 KCL 格式化工具的使用方式。

## 使用方式

* 单文件格式化

```text
kcl-fmt your_config.k
```

* 文件夹内多文件格式化

```text
kcl-fmt your_config_path -R
```

* 命令行参数
  * `-R|--recursive` 设置是否递归遍历子文件夹
  * `-w|--fmt-output` 设置是否输出到标准输出流，不加 `-w` 表示原地格式化 KCL 文件

## 格式化文件效果展示

* 格式化前

```py
import     math
mixin DeploymentMixin:
    service:str ="my-service"
schema DeploymentBase:
    name: str
    image  : str
schema Deployment[replicas] ( DeploymentBase )   :
    mixin[DeploymentMixin]
    replicas   : int   = replicas
    command: [str  ]
    labels: {str:  str}
deploy = Deployment(replicas = 3){}
```

* 格式化后

```py
import math

mixin DeploymentMixin:
    service: str = "my-service"

schema DeploymentBase:
    name: str
    image: str

schema Deployment[replicas](DeploymentBase):
    mixin [DeploymentMixin]
    replicas: int = replicas
    command: [str]
    labels: {str:str}

deploy = Deployment(replicas=3) {}

```
