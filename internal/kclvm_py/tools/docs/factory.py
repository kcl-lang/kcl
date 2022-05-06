import io
import kclvm.kcl.error as kcl_error
import kclvm.tools.docs.templater as templater

# ---------------------------------------------------
# Constants
# ---------------------------------------------------


INVALID_FORMAT_MSG = "an unsupported format, expected {}"


# ---------------------------------------------------
# Factory
# ---------------------------------------------------


class IOFactory:
    def __init__(self):
        self._creators = {}

    def register_format(self, format: str, creator):
        self._creators[format] = creator

    def get(self, format: str, io: io.TextIOBase):
        creator = self._creators.get(format)
        if not creator:
            formats = ",".join(self._creators.keys())
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE,
                arg_msg=INVALID_FORMAT_MSG.format(formats),
            )
        return creator(io)


class PathResolverFactory:
    def __init__(self):
        self._resolvers = {}

    def register_format(self, format: str, resolver):
        self._resolvers[format] = resolver

    def get(self, format: str, doc_name_formatter):
        resolver = self._resolvers.get(format)
        if not resolver:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE,
                arg_msg=INVALID_FORMAT_MSG.format(",".join(self._resolvers.keys())),
            )
        return resolver(
            doc_name_formatter=doc_name_formatter,
            schema_section_render=templater.md_schema_section_render,
        )  # todo, get templater from factory


class DocEscaperFactory:
    def __init__(self):
        self._escapers = {}

    def register_format(self, format: str, escaper):
        self._escapers[format] = escaper

    def get(self, format: str):
        escaper = self._escapers.get(format)
        if not escaper:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE,
                arg_msg=INVALID_FORMAT_MSG.format(",".join(self._escapers.keys())),
            )
        return escaper()
