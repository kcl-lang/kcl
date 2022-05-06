class KCLDocFormat:
    """
    KCL document formats including yaml, json, markdown, reST, HTML5, etc.
    TODO: RST, HTML5 style document generation.
    """

    YAML: str = "YAML"
    JSON: str = "JSON"
    MARKDOWN: str = "MARKDOWN"

    MAPPING = {
        YAML: YAML,
        JSON: JSON,
        MARKDOWN: MARKDOWN,
    }


class KCLDocSuffix:
    """
    KCL document suffix including .yaml, .json, .md, .rst, .html, etc.
    TODO: RST, HTML5 style document generation.
    """

    YAML: str = ".yaml"
    JSON: str = ".json"
    MARKDOWN: str = ".md"
    TO_SUFFIX = {
        KCLDocFormat.YAML: YAML,
        KCLDocFormat.JSON: JSON,
        KCLDocFormat.MARKDOWN: MARKDOWN,
    }


class KCLI18NFormat:
    """
    KCL i18n meta file formats including yaml, json.
    """

    YAML: str = KCLDocFormat.YAML
    JSON: str = KCLDocFormat.JSON
    MAPPING = {
        YAML: YAML,
        JSON: JSON,
    }
    FROM_SUFFIX = {KCLDocSuffix.YAML: YAML, KCLDocSuffix.JSON: JSON}
