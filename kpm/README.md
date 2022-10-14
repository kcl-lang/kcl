# kpm - a KCL Package Manager

## 安装
直接下载在在环境变量path的目录配置好权限即可，如果需要修改配置则根据文档设置即可
## 核心工作逻辑与约定
### 约定 
导入包通过 import external.pkgname.* 的格式导入
### 工作逻辑
将依赖通过嵌套目录的形式，将依赖复制进external文件夹

