"""
https://microsoft.github.io/language-server-protocol/specification#textDocument_documentSymbol
DocumentSymbol[] which is a hierarchy of symbols found in a given text document.
class DocumentSymbol(Model):
    name: str 	                                       # The name of this symbol. Will be displayed in the user interface
                                                         and therefore must not be an empty string or a string only
                                                         consisting of white spaces.
    kind: SymbolKind                                   # The kind of this symbol.
    range: Range                                       # The range enclosing this symbol not including leading/trailing
                                                         whitespace but everything else like comments. This information
                                                         is typically used to determine if the clients cursor is inside
                                                         the symbol to reveal in the symbol in the UI.
    selection_range: Range                             # The range that should be selected and revealed when this symbol
                                                         is being picked, e.g. the name of a function. Must be contained
                                                         by the `range`.
    detail: Optional[str] = None                       # More detail for this symbol, e.g the signature of a function.
    children: Optional[List['DocumentSymbol']] = None  # Children of this symbol, e.g. properties of a class.
    deprecated: Optional[bool] = False                 # Indicates if this symbol is deprecated.

In KCl, we select variables and schema definitions as symbols，, and the schema attributes and mixins in the schema will
be the child nodes of the schema, e.g:

a = b = 1                        a
schema Person:                   b
    mixin [                      Person
        nameMixin                  mixin
    ]                       ->       nameMixin
    age: int = 1                   age
                                 person
person = Person{
    ...
}
"""
from typing import List
from pygls.lsp.types.language_features.document_symbol import DocumentSymbol, SymbolKind
from pygls.lsp.types.basic_structures import Range, Position

import kclvm.kcl.ast as ast
from kclvm.tools.langserver.common import file_to_ast


def range_check(s: DocumentSymbol):
    """
    DocumentSymbol.selection_range must be contained by the DocumentSymbol.range
        a = { ... }
        ^^        ^
        ││        └-range_end
        │└---selection_range_end
        │
        range_start, selection_range_start

    """
    assert isinstance(s, DocumentSymbol)
    range = s.range
    selection_range = s.selection_range
    assert (
        (range.start.line <= selection_range.start.line)
        & (selection_range.start.line <= selection_range.end.line)
        & (selection_range.end.line <= range.end.line)
    )
    if range.start.line == selection_range.start.line:
        assert selection_range.start.character >= range.start.character
    if selection_range.end.line == range.end.line:
        assert selection_range.start.character >= range.start.character
    if s.children:
        for child in s.children:
            range_check(child)


def ast_position_to_range(node: ast.AST) -> Range:
    assert isinstance(node, ast.AST)
    return Range(
        start=Position(line=node.line - 1, character=node.column - 1),
        end=Position(line=node.end_line - 1, character=node.end_column - 1),
    )


def identifier_to_document_symbol(node: ast.Identifier) -> DocumentSymbol:
    assert isinstance(node, ast.Identifier)
    range = Range(
        start=Position(line=node.line - 1, character=node.column - 1),
        end=Position(line=node.end_line - 1, character=node.end_column - 1),
    )
    return DocumentSymbol(
        name=".".join(node.names),
        range=range,
        selection_range=range,
        kind=SymbolKind.Variable,
    )


def schema_attr_to_document_symbol(node: ast.SchemaAttr) -> DocumentSymbol:
    assert isinstance(node, ast.SchemaAttr)
    return DocumentSymbol(
        name=node.name,
        range=ast_position_to_range(node),
        kind=SymbolKind.Property,
        selection_range=ast_position_to_range(node.name_node),
    )


def schema_stmt_to_document_symbol(node: ast.SchemaStmt) -> DocumentSymbol:
    assert isinstance(node, ast.SchemaStmt)
    symbol = DocumentSymbol(
        name=node.name,
        kind=SymbolKind.Struct,
        range=ast_position_to_range(node),
        selection_range=ast_position_to_range(node.name_node),
        children=[],
    )
    if len(node.mixins):
        range = Range(
            start=Position(line=node.line, character=4),
            end=Position(line=node.line, character=9),
        )
        symbol.children.append(
            DocumentSymbol(
                name="mixin",
                kind=SymbolKind.Property,
                range=range,
                selection_range=range,
                children=[identifier_to_document_symbol(id) for id in node.mixins],
            )
        )
    symbol.children += [
        schema_attr_to_document_symbol(attr) for attr in node.GetAttrList()
    ]
    return symbol


def assign_stmt_to_document_symbol(node: ast.AssignStmt) -> DocumentSymbol:
    assert isinstance(node, ast.AssignStmt)
    id_symbols = []
    for identifier in node.targets:
        symbol = identifier_to_document_symbol(identifier)
        symbol.range = ast_position_to_range(node)
        id_symbols.append(symbol)
    return id_symbols


def document_symbol(file: str, code: str = None) -> List[DocumentSymbol]:
    symbols = []
    module = file_to_ast(file, code)
    if not module or not isinstance(module, ast.Module):
        return []
    for stmt in module.body or []:
        if isinstance(stmt, ast.SchemaStmt):
            symbols.append(schema_stmt_to_document_symbol(stmt))
        if isinstance(stmt, ast.AssignStmt):
            symbols.extend(assign_stmt_to_document_symbol(stmt))
    for s in symbols:
        range_check(s)
    return symbols
