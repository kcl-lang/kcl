# KCL 文档生成工具

Kusion 命令行工具支持从 KCL 源码中一键提取模型文档，并支持丰富的输出格式：JSON，YAML 和 Markdown 等。本文介绍 KCL 语言的文档规范，举例说明如何使用 KCL 文档生成工具提取文档，并展示新增本地化语言文档的流程。

## KCL 语言的文档规范

KCL文件的文档主要包含如下两个部分：

* 当前 KCL Moudle 的文档：对当前 KCL 文件的说明
* KCL 文件内包含的所有 Schema 的文档：对当前 Schema 的说明，其中包含 Schema 描述、Schema 各属性的描述、Examples 三部分，具体格式如下：

1. Schema 描述

```python
"""这是Schema一个简短的描述信息
"""
```

2. Schema 各属性的描述：包含属性描述、属性类型、默认值、是否可选

```python
"""
Attributes
----------
x : type, default is a, optional.
    Description of parameter `x`.
y : type, default is b, required.
    Description of parameter `y`.
"""
```

其中，使用 `----------` 表示 `Attributes` 为一个标题(`-` 符号长度与标题长度保持一致)，属性名称与属性类型用冒号 `:` 分隔，属性的说明另起一行并增加缩进进行书写。属性的默认值说明跟在属性类型之后使用逗号 `,` 分隔，书写为 `default is {默认值}` 形式，此外需要说明属性是否为可选/必选，对于可选属性在默认值之后书写 `optional`，对于必选属性在默认值之后书写 `required`。


3. Examples

```python
"""
Examples
--------
val = Schema {
    name = "Alice"
    age = 18
}
"""
```

此外，KCL 文档字符串语法应采用 [re-structured text (reST)](https://docutils.sourceforge.io/rst.html) 语法子集，并使用 [Sphinx](https://www.sphinx-doc.org/en/master/) 渲染呈现。

## 从 KCL 源码生成文档

使用 kcl-doc generate 命令，从用户指定的文件或目录中提取文档，并输出到指定目录。

* 参数说明
```
usage: kcl-doc generate [-h] [--format YAML] [-o OUTPUT] [--r]
                        [--i18n-locale LOCALE] [--repo-url REPO_URL]
                        [files [files ...]]

positional arguments:
  files                 KCL file paths. If there's more than one files to
                        generate, separate them by space

optional arguments:
  -h, --help            show this help message and exit
  --format YAML         Doc file format, support YAML, JSON and MARKDOWN.
                        Defaults to MARKDOWN
  -o OUTPUT, --output-path OUTPUT
                        Specify the output directory. Defaults to ./kcl_doc
  --r, -R, --recursive  Search directory recursively
  --i18n-locale LOCALE  I18n locale, e.g.: zh, zh_cn, en, en_AS. Defaults to
                        en
  --repo-url REPO_URL   The source code repository url. It will displayed in
                        the generated doc to link to the source code.
  --i18n-path I18N_PATH
                        The i18n input file path. It can be a path to an i18n
                        file when generating doc for a single kcl file, or a
                        path to a directory that contains i18n files when
                        generating docs for multipule kcl files. The program
                        will search for the i18n input file according to the
                        locale when generating docs. If i18n file exists, use
                        it instead of source file to generate the doc
```

* 从指定的一个或多个文件中提取文档，并输出到指定目录

```text
kcl-doc generate your_config.k your_another_config.k -o your_docs_output_dir
```

* 从指定目录内，递归地查找 KCL 源码文件，并提取文档

```text
kcl-doc generate your_config_dir -r -o your_docs_output_dir
```

* 在生成文档时，指定源码仓库地址。一经指定，生成的文档中将包含指向源码文件的链接

```text
kcl-doc generate your_config.k -o your_docs_output_dir --repo-url https://url/to/source_code
```

## 新增本地化语言的文档

如前所示，默认情况下，文档生成工具提取的文档以源码 docstring 的内容为准，因而文档的语言随 docstring 编写语言而定。如果需要为源文件新增本地化语言的文档，则可以遵循按如下步骤：

1. 初始化 i18n 配置文件。该步骤基于指定的 KCL 源码文件，生成相应的 i18n 配置文件，文件格式可选 JSON/YAML，默认为 YAML. 输出的配置文件名称将以指定的目标本地化方言结尾

```text
kcl-doc init-i18n your_config.k --format JSON --i18n-locale your_target_locale
```

2. 手动修改上述生成的 i18n 配置文件，使用目标语言修改配置中的 doc 字段

3. 基于修改后的 i18n 配置，生成本地化语言的文档。工具将查找指定目标语言的 i18n 配置文件，并转化为最终的文档

```text
kcl-doc generate your_config_dir --i18n-locale your_target_locale --format Markdown
```

接下来，通过一个小例子演示新增本地化语言文档的过程。

1. 准备 KCL 源码文件，例如 server.k：

```python
schema Server:
    """Server is the common user interface for long-running
    services adopting the best practice of Kubernetes.

    Attributes
    ----------
    workloadType : str, default is "Deployment", required
        Use this attribute to specify which kind of long-running service you want.
        Valid values: Deployment, CafeDeployment.
        See also: kusion_models/core/v1/workload_metadata.k.
    name : str, required
        A Server-level attribute.
        The name of the long-running service.
        See also: kusion_models/core/v1/metadata.k.
    labels : {str:str}, optional
        A Server-level attribute.
        The labels of the long-running service.
        See also: kusion_models/core/v1/metadata.k.

    Examples
    ----------------------
    myCustomApp = AppConfiguration {
        name = "componentName"
    }
    """

    workloadType: str = "Deployment"
    name: str
    labels?: {str: str}
```

2. 从 server.k 得到初始化的 i18n 配置文件，例如希望为其增加中文文档，指定生成的配置文件格式为 YAML

```text
kcl init-i18n server.k --format YAML --i18n-locale zh_cn
```

该命令将在当前目录下创建 kcl_doc 目录，并生成 i18n 配置文件 kcl_doc/i18n_server_zh_cn.yaml，其内容如下：

```yaml
name: server
relative_path: ./server.k
schemas:
- name: Server
  doc: |
    Server is the common user interface for long-running
    services adopting the best practice of Kubernetes.
  attributes:
  - name: workloadType
    doc: |
      Use this attribute to specify which kind of long-running service you want.
      Valid values: Deployment, CafeDeployment.
      See also: kusion_models/core/v1/workload_metadata.k.
    type:
      type_str: str
      type_category: BUILTIN
      builtin_type: STRING
    default_value: '"Deployment"'
    is_optional: false
  - name: name
    doc: |
      A Server-level attribute.
      The name of the long-running service.
      See also: kusion_models/core/v1/metadata.k.
    type:
      type_str: str
      type_category: BUILTIN
      builtin_type: STRING
    is_optional: false
    default_value: ''
  - name: labels
    doc: |
      A Server-level attribute.
      The labels of the long-running service.
      See also: kusion_models/core/v1/metadata.k.
    type:
      type_str: '{str: str}'
      type_category: DICT
      dict_type:
        key_type:
          type_str: str
          type_category: BUILTIN
          builtin_type: STRING
        value_type:
          type_str: str
          type_category: BUILTIN
          builtin_type: STRING
    is_optional: true
    default_value: ''
  examples: |
    myCustomApp = AppConfiguration {
        name = "componentName"
    }
doc: ''
source_code_url: ''
```

3. 修改初始化得到的 i18n 配置，将其中的 doc 字段修改为中文的描述，修改后的配置如下：

```yaml
name: server
relative_path: ./server.k
schemas:
- name: Server
  doc: |
    Server 模型定义了采用 Kubernetes 最佳实践的持续运行的服务的通用配置接口
  attributes:
  - name: workloadType
    doc: |
      workloadType 属性定义了服务的类型，是服务级别的属性。合法的取值有：Deployment, CafeDeployment.
      另请查看：kusion_models/core/v1/workload_metadata.k.
    type:
      type_str: str
      type_category: BUILTIN
      builtin_type: STRING
    default_value: '"Deployment"'
    is_optional: false
  - name: name
    doc: |
      name 为服务的名称，是服务级别的属性。
      另请查看：kusion_models/core/v1/metadata.k.
    type:
      type_str: str
      type_category: BUILTIN
      builtin_type: STRING
    is_optional: false
    default_value: ''
  - name: labels
    doc: |
      labels 为服务的标签，是服务级别的属性。
      另请查看：kusion_models/core/v1/metadata.k.
    type:
      type_str: '{str: str}'
      type_category: DICT
      dict_type:
        key_type:
          type_str: str
          type_category: BUILTIN
          builtin_type: STRING
        value_type:
          type_str: str
          type_category: BUILTIN
          builtin_type: STRING
    is_optional: true
    default_value: ''
  examples: |
    myCustomApp = AppConfiguration {
        name = "componentName"
    }
doc: ''
source_code_url: ''
```

4. 基于修改后的 i18n 配置，生成本地化语言的文档，执行如下命令，将输出中文的文档 kcl_doc/doc_server_zh_cn.md，命令及生成的文档内容如下：

```text
kcl-doc generate server.k --i18n-locale zh_cn --format Markdown
```

~~~markdown
# server
## Schema Server
Server 模型定义了采用 Kubernetes 最佳实践的持续运行的服务的通用配置接口

### Attributes
|Name and Description|Type|Default Value|Required|
|--------------------|----|-------------|--------|
|**workloadType**<br />workloadType 属性定义了服务的类型，是服务级别的属性。合法的取值有：Deployment, CafeDeployment.<br />另请查看：kusion_models/core/v1/workload_metadata.k.|str|"Deployment"|**required**|
|**name**<br />name 为服务的名称，是服务级别的属性。<br />另请查看：kusion_models/core/v1/metadata.k.|str|Undefined|**required**|
|**labels**<br />labels 为服务的标签，是服务级别的属性。<br />另请查看：kusion_models/core/v1/metadata.k.|{str: str}|Undefined|optional|
### Examples
```
myCustomApp = AppConfiguration {
    name = "componentName"
}
```

<!-- Auto generated by kcl-doc tool, please do not edit. -->

~~~

## 附录

### 常见的 reST 概念

对于 reST 格式的文档，段落和缩进很重要，新段落用空白行标记，缩进即为表示输出中的缩进。可以使用如下方式表示字体样式：

* \*斜体\*
* \*\*粗体\*\*
* \`\`等宽字体\`\`

参考 [reST 文档](https://docutils.sourceforge.io/rst.html)获得更多帮助。
