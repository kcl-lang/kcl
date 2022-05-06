from dataclasses import dataclass
from enum import Enum
from typing import Optional, List, Dict, Tuple

import kclvm.kcl.info as kcl_info

from kclvm.compiler.extension.builtin import BUILTIN_FUNCTIONS


class SymbolScope(Enum):
    """Symbol Scope

    Parameters
    ----------
    GLOBAL: str
        Top level variables including variables in IfStmt

    LOCAL: str
        Variables in schema context or in for comprehension

    BUILTIN: str
        Built-in variables e.g., `print`, `sum`,

    FREE: str
        Variables that need to be captured by closures

    INTERNAL: str
        Internal variables used only for record names e.g., `b` and `c` in `a.b.c`
    """

    GLOBAL = "GLOBAL"
    LOCAL = "LOCAL"
    BUILT_IN = "BUILT_IN"
    FREE = "FREE"
    INTERNAL = "INTERNAL"


@dataclass
class Symbol:
    """Symbol

    Parameters
    ----------
    name: str
        The symbol name

    index: int
        The symbol index in the symbol table

    scope:
        The symbol scope

    define_count:
        The number of times the symbol is defined or declared
    """

    name: str
    index: int
    scope: SymbolScope = None
    define_count: int = 0


@dataclass
class SymbolTable:
    """Symbol table

    Parameters
    ----------
    outer: SymbolTable
        The symbol table in the outer symbol scope
        e.g., The schema context symbol table outer is the top level scope symbol table.

    store: Dict[str, Symbol]
        The current scope symbol map to store all symbols

    free_symbols: List[Symbol]
        The free symbol list in the current scope

    num_definitions: int
        The total number of symbols
    """

    outer: "SymbolTable" = None
    store: Dict[str, Symbol] = None
    free_symbols: List[Symbol] = None
    num_definitions: int = 0

    def define(self, name: str, scope: SymbolScope = None) -> Tuple[Symbol, bool]:
        """
        Define a symbol named 'name' with 'scope' and put it into symbol table
        """
        assert isinstance(name, str)

        def default_scope():
            """The inner default symbol scope

            If outer is exist, return LOCAL scope, else return GLOBAL scope
            """
            return SymbolScope.GLOBAL if self.outer is None else SymbolScope.LOCAL

        symbol = Symbol(
            name=name, index=self.num_definitions, scope=scope or default_scope()
        )
        is_exist = False
        if (
            name in self.store
            and not kcl_info.isprivate_field(name)
            and self.store[name].scope == SymbolScope.GLOBAL
        ):
            is_exist = True
            self.num_definitions += 1
            return self.store[name], is_exist
        # Internal scope variable skip exist symbol
        if symbol.scope == SymbolScope.INTERNAL and name in self.store:
            pass
        else:
            self.store[name] = symbol
        self.num_definitions += 1
        return symbol, is_exist

    def define_builtin(self, name: str, index: int) -> Symbol:
        """Define a builtin function object"""
        if self.store is None or not isinstance(self.store, dict):
            raise Exception("Invalid symbol table store")
        symbol = Symbol(name=name, index=index)
        symbol.scope = SymbolScope.BUILT_IN
        self.store[name] = symbol
        return symbol

    def define_free(self, original: Symbol) -> Symbol:
        """
        Define a symbol named 'name' with free scope and put it into symbol table
        """
        self.free_symbols.append(original)
        self.store[original.name] = Symbol(
            name=original.name, index=original.index, scope=SymbolScope.FREE
        )
        return self.store[original.name]

    def register_builtins(self):
        """Register builtin functions into symbol table"""
        builtins = BUILTIN_FUNCTIONS
        for i, builtin in enumerate(builtins):
            self.define_builtin(builtin, i)
        return self

    def resolve(self, name: str) -> Optional[Symbol]:
        """Resolve a symbol named 'name'

        Search from the current scope, if not found, search from the symbol table of its outer
        """
        obj = self.store.get(name)
        if not obj and self.outer:
            obj = self.outer.resolve(name)
            if not obj:
                return obj
            if obj.scope == SymbolScope.GLOBAL or obj.scope == SymbolScope.BUILT_IN:
                return obj
            elif obj.scope == SymbolScope.INTERNAL:
                return None
            return self.define_free(obj)
        return obj

    def delete(self, name: str, scope: SymbolScope):
        """Delete name from the symbol table"""
        if (
            not name
            or not scope
            or name not in self.store
            or self.store[name].scope != scope
        ):
            return
        del self.store[name]

    # Static methods

    @staticmethod
    def new(outer=None, num=0):
        """New an empty symbol table"""
        return SymbolTable(
            outer=outer,
            store={},
            free_symbols=[],
            num_definitions=num,
        )

    @staticmethod
    def new_with_built_in(outer=None, num=0):
        """New a symbol table with all builtin functions"""
        return SymbolTable.new(outer, num).register_builtins()
