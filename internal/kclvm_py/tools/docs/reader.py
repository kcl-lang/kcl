import ruamel.yaml as yaml
import json
import io

from abc import ABC, abstractmethod

import kclvm.tools.docs.pb as pb
import kclvm.tools.docs.model_pb2 as model
import kclvm.tools.docs.factory as factory
import kclvm.tools.docs.formats as doc_formats

# ---------------------------------------------------
# Constants
# ---------------------------------------------------


ENDLINE = "\n"


# -------------------------------------------------------
# Document reader from KCL ModuleDoc type ot markdown type
# -------------------------------------------------------


class ModuleDocReader(ABC):
    """Module document reader class.

    This class provides basic functions for reader KCL Module from the IO.
    """

    def __init__(self, io_in: io.TextIOBase):
        self.io_in: io.TextIOBase = io_in

    @abstractmethod
    def read_doc(self) -> model.ModuleDoc:
        pass

    def read(self) -> str:
        return self.io_in.read()


class JsonDocReader(ModuleDocReader):
    """JSON document reader based on ModuleDocReader.

    The class is used to read KCL module document from JSON file.
    """

    def _load_module_from_json(self, data) -> model.ModuleDoc:
        return pb.FromJson(data, model.ModuleDoc())

    def read_doc(self) -> model.ModuleDoc:
        json_data = self.read()
        return self._load_module_from_json(json_data)


class YamlDocReader(JsonDocReader):
    """YAML document reader based on JsonDocReader.

    The class is used to read KCL module document from YAML file
    """

    def read_doc(self) -> model.ModuleDoc:
        dict_data = yaml.safe_load(self.io_in)
        json_data = json.dumps(dict_data)
        return self._load_module_from_json(json_data)


class MarkDownDocReader(YamlDocReader):
    """Markdown document reader based on KCLModuleDocReader.

    The class is used to read KCL module document from markdown file.
    """

    def read_doc(self) -> model.ModuleDoc:
        """TODO: Parse a Markdown string to KCL module document model."""
        raise NotImplementedError()


factory = factory.IOFactory()
factory.register_format(doc_formats.KCLDocFormat.JSON, JsonDocReader)
factory.register_format(doc_formats.KCLDocFormat.YAML, YamlDocReader)
factory.register_format(doc_formats.KCLDocFormat.MARKDOWN, MarkDownDocReader)
