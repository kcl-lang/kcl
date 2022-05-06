from ruamel.yaml import YAML
import io

from abc import ABC, abstractmethod

import kclvm.tools.docs.pb as pb
import kclvm.tools.docs.model_pb2 as model
import kclvm.tools.docs.factory as factory
import kclvm.tools.docs.formats as doc_formats
import kclvm.tools.docs.templater as templater


# ---------------------------------------------------
# Constants
# ---------------------------------------------------


ENDLINE = "\n"


# -------------------------------------------------------
# Document writer from KCL ModuleDoc type ot markdown type
# -------------------------------------------------------


class ModuleDocWriter(ABC):
    """Module document writer class.

    This class provides basic functions for write KCL Module to the IO.
    """

    def __init__(self, out: io.TextIOBase):
        self.out: io.TextIOBase = out

    @abstractmethod
    def write_doc(self, module: model.ModuleDoc) -> None:
        pass

    def write(self, content: str = ""):
        """Write the content to the string io"""
        self.out.write(content)

    def writeln(self, content: str = ""):
        """Write the content with an endline to the string io"""
        self.write(content + ENDLINE)


class JsonDocWriter(ModuleDocWriter):
    """JSON document writer based on ModuleDocWriter.

    The class is used to generate JSON file from KCL module document string.
    """

    def _dump_json_str(self, module: model.ModuleDoc) -> str:
        return pb.ToJson(module)

    def write_doc(self, module: model.ModuleDoc) -> None:
        json_str = self._dump_json_str(module)
        self.write(json_str)


class YamlDocWriter(JsonDocWriter):
    """YAML document writer based on JsonDocWriter.

    The class is used to generate YAML file from KCL module document string.
    """

    def write_doc(self, module: model.ModuleDoc) -> None:
        def set_style(d, flow):
            if isinstance(d, dict):
                if flow:
                    d.fa.set_flow_style()
                else:
                    d.fa.set_block_style()
                for k in d:
                    set_style(d[k], flow)
            elif isinstance(d, list):
                if flow:
                    d.fa.set_flow_style()
                else:
                    d.fa.set_block_style()
                for item in d:
                    set_style(item, flow)

        json_str = self._dump_json_str(module)
        yaml = YAML()
        data = yaml.load(json_str)
        set_style(data, flow=False)
        yaml.dump(data, self.out)


class MarkDownDocWriter(ModuleDocWriter):
    """Markdown document writer based on KCLModuleDocWriter.

    The class is used to generate markdown file from KCL module document string.
    """

    def write_doc(self, module: model.ModuleDoc) -> None:
        """
        Write document
        """
        self.writeln(templater.md_module_doc_templater(module))


factory = factory.IOFactory()
factory.register_format(doc_formats.KCLDocFormat.JSON, JsonDocWriter)
factory.register_format(doc_formats.KCLDocFormat.YAML, YamlDocWriter)
factory.register_format(doc_formats.KCLDocFormat.MARKDOWN, MarkDownDocWriter)
