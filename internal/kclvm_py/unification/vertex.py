# Copyright 2021 The KCL Authors. All rights reserved.

from io import StringIO
from dataclasses import dataclass, field
from typing import cast, List, Optional, Union

import kclvm.kcl.ast as ast
import kclvm.compiler.astutil as astutil


VERTEX_ROOT_NAME = "@root"


@dataclass
class Meta:
    """The filename, line and column info"""

    filename: Optional[str] = field(default_factory=lambda: None)
    line: Optional[int] = field(default_factory=lambda: None)
    column: Optional[int] = field(default_factory=lambda: None)
    end_line: Optional[int] = field(default_factory=lambda: None)
    end_column: Optional[int] = field(default_factory=lambda: None)

    @staticmethod
    def from_ast_node(node: ast.AST) -> "Meta":
        return (
            Meta(
                filename=node.filename,
                line=node.line,
                column=node.column,
                end_line=node.end_line,
                end_column=node.end_column,
            )
            if node and isinstance(node, ast.AST)
            else Meta()
        )


@dataclass
class ConfigMeta:
    """The schema config name, args and kwargs AST node references"""

    name: str = None
    pkgpath: str = None
    args: List[ast.Expr] = None
    kwargs: List[ast.Keyword] = None


@dataclass
class VertexOption:
    """The vertex unification options"""

    is_union: bool = False  # `:`
    is_override: bool = False  # `=`
    is_append: bool = False  # `+=`
    is_unique: bool = False  # `!`
    is_strategy_merge: bool = False  # `?`


@dataclass
class Vertex:
    """
    Parameters
    ----------
    name: Union[int, float, str, ast.AST]
        Vertex node name. When the name is a variable needed to be calculated in VM,
        it is a AST type
    adjs: List["Vertex"]
        The list of downstream nodes of the vertex
    node: ast.AST
        The AST node reference
    meta: Mate
        The filename, line and column info
    config_meta: ConfigMeta:
        The schema config name, args and kwargs AST node references
    option:
        The vertex unification options

    Methods
    -------
    vertex_to_ast:
        Vertex -> AST
    ast_to_vertex:
        AST -> Vertex
    """

    name: Union[int, float, str, ast.AST]
    adjs: Optional[List["Vertex"]]
    node: ast.AST = field(default_factory=lambda: None)
    meta: Optional[Meta] = field(default_factory=lambda: Meta)
    config_meta: Optional[ConfigMeta] = field(default_factory=lambda: ConfigMeta)
    option: Optional[VertexOption] = field(default_factory=lambda: VertexOption)

    # ---------------
    # Member method
    # ---------------

    def pretty(self, indent: int = 1) -> str:
        """Pretty print to show the vertex structure"""
        with StringIO() as buf:
            buf.write("name: " + str(self.name) + "\n")
            buf.write("is_unique: " + str(self.option.is_unique) + "\n")
            if self.config_meta.name:
                buf.write("config_name: " + self.config_meta.name + "\n")
            if self.adjs:
                buf.write("adjs: \n")
                for v in self.adjs:
                    lines = v.pretty(indent).split("\n")
                    buf.write(
                        "\n".join([" " * indent * 4 + line for line in lines]) + "\n"
                    )
            else:
                buf.write("value: " + str(self.node))
            return buf.getvalue().rstrip(" \n") + "\n"

    def vertex_to_ast(self) -> Optional[ast.AST]:
        """Vertex to KCL AST"""

        def append_config_key_value(
            t: Union[ast.SchemaExpr, ast.ConfigExpr],
            key: Union[int, float, str, Optional[ast.AST]],
            value: ast.AST,
            op: int = ast.ConfigEntryOperation.UNION,
            meta: Meta = Meta(),
        ):
            if isinstance(t, ast.SchemaExpr):
                if not t.config:
                    t.config = ast.ConfigExpr(line=t.line, column=t.column)
                    t.config.filename = meta.filename
                if not t.config.items:
                    t.config.items = []
                # If `key` is None, it may be a double star expr
                key_node = None
                if key and isinstance(key, ast.AST):
                    key_node = key
                elif isinstance(key, str):
                    key_node = ast.Identifier(
                        line=meta.line, column=meta.column, names=[key]
                    ).set_filename(meta.filename)
                    key_node.end_line, key_node.end_column = (
                        meta.end_line,
                        meta.end_column,
                    )
                elif isinstance(key, (int, float)):
                    key_node = ast.NumberLit(
                        line=meta.line, column=meta.column, value=key
                    )
                    key_node.end_line, key_node.end_column = (
                        meta.end_line,
                        meta.end_column,
                    )
                if isinstance(key_node, ast.AST):
                    key_node.filename = meta.filename
                value = cast(ast.Expr, value)
                t.config.items.append(
                    ast.ConfigEntry(
                        key=key_node,
                        value=value,
                        operation=op,
                    )
                )
            elif isinstance(t, ast.ConfigExpr):
                if not t.items:
                    t.items = []
                # If `key` is None, it may be a double star expr
                key_node = None
                if key and isinstance(key, ast.AST):
                    key_node = key
                elif isinstance(key, str):
                    key_node = ast.StringLit(
                        line=meta.line, column=meta.column, value=key
                    )
                    key_node.end_line, key_node.end_column = (
                        meta.end_line,
                        meta.end_column,
                    )
                elif isinstance(key, (int, float)):
                    key_node = ast.NumberLit(
                        line=meta.line, column=meta.column, value=key
                    )
                    key_node.end_line, key_node.end_column = (
                        meta.end_line,
                        meta.end_column,
                    )
                if isinstance(key_node, ast.AST):
                    key_node.filename = meta.filename
                key_node = cast(ast.Expr, key_node)
                t.items.append(
                    ast.ConfigEntry(
                        key=key_node,
                        value=value,
                        operation=op,
                    )
                )

        # Get root vertex
        if isinstance(self.name, str) and self.name == VERTEX_ROOT_NAME:
            module = ast.Module(filename=self.meta.filename, line=1, column=1)
            for v in self.adjs:
                assign_stmt = ast.AssignStmt(
                    line=v.meta.line,
                    column=v.meta.column,
                )
                assign_stmt.targets = [
                    ast.Identifier(
                        line=v.node.line,
                        column=v.node.column,
                        names=[v.name],
                        ctx=ast.ExprContext.STORE,
                    ).set_filename(v.node.filename)
                ]
                assign_stmt.end_line = v.meta.end_line
                assign_stmt.end_column = v.meta.end_column
                assign_stmt.value = v.vertex_to_ast()
                assign_stmt.filename = v.meta.filename
                module.body.append(assign_stmt)
            return module
        # Get normal vertex such as in the right assignment
        else:
            # SchemaExpr config
            if self.config_meta.name:
                config_expr = ast.SchemaExpr(
                    line=self.meta.line,
                    column=self.meta.column,
                )
                name = ast.Identifier(
                    line=self.meta.line,
                    column=self.meta.column,
                    names=self.config_meta.name.split("."),
                ).set_filename(self.meta.filename)
                name.end_line, name.end_column = (
                    self.meta.end_line,
                    self.meta.end_column,
                )
                config_expr.name = name
                config_expr.name.pkgpath = self.config_meta.pkgpath
                config_expr.args = self.config_meta.args
                config_expr.kwargs = self.config_meta.kwargs
            # ConfigExpr config
            else:
                config_expr = ast.ConfigExpr(
                    line=self.meta.line,
                    column=self.meta.column,
                )
            config_expr.end_line, config_expr.end_column = (
                self.meta.end_line,
                self.meta.end_column,
            )
            if self.adjs:
                for vv in self.adjs:
                    op = ast.ConfigEntryOperation.UNION
                    op = (
                        ast.ConfigEntryOperation.OVERRIDE
                        if vv.option.is_override
                        else op
                    )
                    op = ast.ConfigEntryOperation.INSERT if vv.option.is_append else op
                    if vv.adjs:
                        append_config_key_value(
                            config_expr, vv.name, vv.vertex_to_ast(), op, vv.meta
                        )
                    elif vv.node:
                        append_config_key_value(
                            config_expr, vv.name, vv.node, op, vv.meta
                        )
                return config_expr
            elif self.node:
                return self.node
            return None

    # ---------------
    # Static method
    # ---------------

    @staticmethod
    def update_vertex_option(v: "Vertex", op: int):
        """Update the vertex option using the schema config operation"""
        if not isinstance(v, Vertex):
            return
        v.option = VertexOption(
            is_override=op == ast.ConfigEntryOperation.OVERRIDE,
            is_append=op == ast.ConfigEntryOperation.INSERT,
            is_union=op == ast.ConfigEntryOperation.UNION,
        )

    @staticmethod
    def ast_to_vertex(
        t: ast.AST,
        name: Optional[Union[int, float, str, ast.AST]] = None,
        is_in_schema: bool = False,
    ) -> Optional["Vertex"]:
        """Build a vertex from AST"""
        if not t or not isinstance(t, ast.AST):
            return None
        if isinstance(t, ast.Module):
            t = cast(ast.Module, t)
            root = Vertex.new_root(
                node=t,
                adjs=[],
            )
            declarations = astutil.filter_declarations(t)
            for d in declarations:
                if "." not in d.name:
                    vertex = Vertex.ast_to_vertex(d.value, d.name)
                    vertex.meta.filename = d.filename
                    vertex.option.is_union = d.is_union
                    root.adjs.append(vertex)
            return root
        elif isinstance(t, ast.SchemaExpr):
            vertex = Vertex.ast_to_vertex(t.config, name, True)
            vertex.meta = Meta.from_ast_node(t)
            vertex.name = name
            vertex.config_meta = ConfigMeta(
                name=t.name.get_name(),
                pkgpath=t.name.pkgpath,
                args=t.args,
                kwargs=t.kwargs,
            )
            if not vertex.adjs and isinstance(vertex.node, ast.ConfigExpr):
                vertex.node = t
            return vertex
        elif isinstance(t, ast.ConfigExpr):
            vertex = Vertex(
                node=t,
                name=name,
                adjs=[],
                meta=Meta.from_ast_node(t),
            )
            for key, value, operation in zip(
                t.keys,
                t.values,
                t.operations
                if isinstance(t, ast.ConfigExpr)
                else [ast.ConfigEntryOperation.UNION] * len(t.keys),
            ):
                # Double star expression
                if not key:
                    value_vertex = Vertex.ast_to_vertex(
                        value, name=None, is_in_schema=is_in_schema
                    )
                    value_vertex.node = value
                    value_vertex.meta = Meta.from_ast_node(value)
                    Vertex.update_vertex_option(value_vertex, operation)
                    vertex.adjs.append(value_vertex)
                elif isinstance(key, ast.Identifier) and (
                    isinstance(t, ast.ConfigExpr) or is_in_schema
                ):
                    nest_key_len = len(key.names)
                    nest_vertex_list = [
                        Vertex(
                            name=key.names[i],
                            node=key,
                            adjs=[],
                            meta=Meta.from_ast_node(key),
                        )
                        for i in range(nest_key_len - 1)
                    ]
                    final_vertex = Vertex.ast_to_vertex(
                        value, name=key.names[-1], is_in_schema=is_in_schema
                    )
                    final_vertex.meta = Meta.from_ast_node(key)
                    if nest_key_len > 1:
                        # Link all vertex in nest vertex list
                        for i in range(nest_key_len - 2):
                            nest_vertex_list[i].adjs = [nest_vertex_list[i + 1]]
                        nest_vertex_list[-1].adjs = [final_vertex]
                    value_vertex = (
                        final_vertex if nest_key_len == 1 else nest_vertex_list[0]
                    )
                    value_vertex.meta = Meta.from_ast_node(key)
                    if isinstance(value, ast.SchemaExpr):
                        value_vertex.config_meta = ConfigMeta(
                            name=value.name.get_name(),
                            args=value.args,
                            kwargs=value.kwargs,
                        )
                    Vertex.update_vertex_option(
                        value_vertex, ast.ConfigEntryOperation.UNION
                    )
                    Vertex.update_vertex_option(final_vertex, operation)
                    vertex.adjs.append(value_vertex)
                else:
                    # Variable attributes that cannot be clearly expressed
                    # Such as string interpolation which denotes a runtime value
                    value_vertex = Vertex.ast_to_vertex(
                        value,
                        name=key.value
                        if isinstance(key, (ast.StringLit, ast.NumberLit))
                        else key,
                        is_in_schema=is_in_schema,
                    )
                    value_vertex.meta = Meta.from_ast_node(key)
                    Vertex.update_vertex_option(value_vertex, operation)
                    vertex.adjs.append(value_vertex)
            return vertex
        # Vertex end node and its adjs is None
        return Vertex(
            name=name,
            node=t,
            adjs=None,
            meta=Meta.from_ast_node(t),
        )

    @staticmethod
    def new_root(
        node: ast.Module = None, adjs: Optional[List["Vertex"]] = None
    ) -> "Vertex":
        """New a empty vertex root node"""
        return Vertex(
            name=VERTEX_ROOT_NAME,
            node=node,
            adjs=adjs,
            meta=Meta(
                filename=node.filename,
                line=1,
                column=1,
                end_line=1,
                end_column=1,
            ),
        )
