import os
import pathlib
from abc import ABC, abstractmethod

import kclvm.tools.docs.factory as factory
import kclvm.tools.docs.formats as doc_formats
import kclvm.tools.docs.model_pb2 as model
import kclvm.tools.docs.utils as utils


class LinkResolver(ABC):
    """
    path resolver refills the path links in Type.type_str
    """

    def __init__(self, doc_name_formatter, schema_section_render):
        self.doc_name_formatter = doc_name_formatter
        self.schema_section_render = schema_section_render

    @abstractmethod
    def resolve(
        self,
        doc: model.ModuleDoc,
        root: str,
        locale: str,
        with_locale_suffix: bool = False,
    ) -> model.ModuleDoc:
        pass


class MarkdownLinkResolver(LinkResolver):
    def resolve(
        self,
        module: model.ModuleDoc,
        root: str,
        locale: str,
        with_locale_suffix: bool = False,
    ) -> model.ModuleDoc:
        def add_link(
            tpe: model.Type, current_module: model.ModuleDoc, root: str
        ) -> str:
            def to_md_anchor(original: str) -> str:
                return "-".join(original.lower().split())

            def resolve_relative_path(to_dir: str, from_dir: str) -> str:
                if to_dir == from_dir:
                    return ""
                common_path = os.path.commonpath([from_dir, to_dir])
                upper_count = len(from_dir.strip("/").split("/")) - len(
                    common_path.strip("/").split("/")
                )

                return (
                    "../" * upper_count
                    + to_dir.replace(common_path, "", 1).lstrip("/")
                    + ("" if to_dir == common_path else "/")
                )

            if tpe.type_category in [
                model.Type.TypeCategory.ANY,
                model.Type.TypeCategory.BUILTIN,
                model.Type.TypeCategory.LIT,
                model.Type.TypeCategory.NAMED_CONSTANT,
            ]:
                return tpe.type_str
            if tpe.type_category == model.Type.TypeCategory.SCHEMA:
                assert tpe.schema_type
                section_link = to_md_anchor(
                    self.schema_section_render(tpe.schema_type.name)
                )
                from_path = pathlib.Path(current_module.relative_path)
                to_path = pathlib.Path(tpe.schema_type.relative_path)

                file_link = ""
                if str(from_path) != str(to_path):
                    # defines in different file
                    relative_dir = resolve_relative_path(
                        to_dir=str(to_path.parent), from_dir=str(from_path.parent)
                    )
                    file_link = f"{relative_dir}{self.doc_name_formatter(utils.module_name(tpe.schema_type.relative_path), locale, doc_formats.KCLDocFormat.MARKDOWN, with_locale_suffix)}"
                    # remove .md file extension suffix
                    file_link = file_link[:-3]
                return f"[{tpe.type_str}]({file_link}#{section_link})"
            if tpe.type_category == model.Type.TypeCategory.UNION:
                assert tpe.union_type
                return " \\| ".join(
                    [
                        add_link(t, current_module=current_module, root=root)
                        for t in tpe.union_type.types
                    ]
                )
            if tpe.type_category == model.Type.TypeCategory.DICT:
                assert tpe.dict_type
                key_type_str = add_link(
                    tpe.dict_type.key_type, current_module=current_module, root=root
                )
                value_type_str = add_link(
                    tpe.dict_type.value_type, current_module=current_module, root=root
                )
                return (
                    f"{{{key_type_str}: {value_type_str}}}"
                    if value_type_str
                    else f"{{{key_type_str}:}}"
                )
            if tpe.type_category == model.Type.TypeCategory.LIST:
                assert tpe.list_type
                return f"[{add_link(tpe.list_type.item_type, current_module=current_module, root=root)}]"
            return tpe.type_str

        for schema in module.schemas:
            for attr in schema.attributes:
                attr.type.type_str = add_link(attr.type, module, root)
            if schema.base_schema:
                schema.base_schema.type_str = add_link(schema.base_schema, module, root)
        return module


class YamlLinkResolver(LinkResolver):
    def resolve(
        self,
        module: model.ModuleDoc,
        root: str,
        locale: str,
        with_locale_suffix: bool = False,
    ) -> model.ModuleDoc:
        return module


class JsonLinkResolver(LinkResolver):
    def resolve(
        self,
        module: model.ModuleDoc,
        root: str,
        locale: str,
        with_locale_suffix: bool = False,
    ) -> model.ModuleDoc:
        return module


factory = factory.PathResolverFactory()
factory.register_format(doc_formats.KCLDocFormat.MARKDOWN, MarkdownLinkResolver)
factory.register_format(doc_formats.KCLDocFormat.YAML, YamlLinkResolver)
factory.register_format(doc_formats.KCLDocFormat.JSON, JsonLinkResolver)
