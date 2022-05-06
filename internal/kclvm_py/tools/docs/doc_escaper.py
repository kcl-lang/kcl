from abc import ABC, abstractmethod

import kclvm.tools.docs.factory as factory
import kclvm.tools.docs.formats as doc_formats
import kclvm.tools.docs.model_pb2 as model


class DocEscaper(ABC):
    """
    doc escaper checks the special character in the doc and escape it
    """

    @abstractmethod
    def escape(self, doc: model.ModuleDoc) -> model.ModuleDoc:
        pass


class MarkdownDocEscaper(DocEscaper):
    def escape(self, module: model.ModuleDoc) -> model.ModuleDoc:
        # remove line breaks at the beginning and the end
        module.doc = self.escape_special_symbol(module.doc.strip("\n"))
        for schema in module.schemas:
            schema.name = self.escape_special_symbol(schema.name)
            schema.doc = self.escape_special_symbol(schema.doc.strip("\n"))
            if schema.attributes:
                for attr in schema.attributes:
                    attr.name = self.escape_special_symbol(attr.name)
                    attr.doc = self.escape_special_symbol(attr.doc.strip("\n"))
                    attr.type.type_str = self.escape_special_symbol(attr.type.type_str)
            if schema.examples:
                schema.examples = schema.examples.strip("\n")
        return module

    @staticmethod
    def escape_special_symbol(name: str) -> str:
        return (
            name.replace("_", "\\_")
            .replace("*", "\\*")
            .replace("#", "\\#")
            .replace("|", "&#124;")
            .replace("<", "\\<")
            .replace(">", "\\>")
            .replace("\n", "<br />")
        )


class YamlDocEscaper(DocEscaper):
    def escape(self, module: model.ModuleDoc) -> None:
        return module


class JsonDocEscaper(DocEscaper):
    def escape(self, module: model.ModuleDoc) -> None:
        return module


factory = factory.DocEscaperFactory()
factory.register_format(doc_formats.KCLDocFormat.MARKDOWN, MarkdownDocEscaper)
factory.register_format(doc_formats.KCLDocFormat.YAML, YamlDocEscaper)
factory.register_format(doc_formats.KCLDocFormat.JSON, JsonDocEscaper)
