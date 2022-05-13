# Copyright 2020 The KCL Authors. All rights reserved.

import typing
import pathlib

from lark import Lark
from lark.indenter import Indenter
from lark.tree import Tree as LarkTree
from lark.lexer import Token as LarkToken

import kclvm.kcl.error as kcl_error
import kclvm.compiler.parser.lark_pb2 as lark_pb

_CACHE_FILE = pathlib.Path(__file__).parent.joinpath("lark_parser.pickle")
START_RULE = "start"

filename = ""


class KCLIndenter(Indenter):
    NL_type = "NEWLINE"
    OPEN_PAREN_types = ["LPAR", "LSQB", "LBRACE"]
    CLOSE_PAREN_types = ["RPAR", "RSQB", "RBRACE"]
    INDENT_type = "_INDENT"
    DEDENT_type = "_DEDENT"
    tab_len = 4
    _indent_has_space = False  # Mark whether there are spaces in an indent_level
    _indent_has_tab = False  # Mark whether there are tabs in an indent_level

    def __init__(self):
        super().__init__()
        self.reset_indent_space_tab()

    def reset_indent_space_tab(self):
        self._indent_has_space = False
        self._indent_has_tab = False

    def process(self, stream):
        self.paren_level = 0
        self.indent_level = [0]
        self.reset_indent_space_tab()
        return self._process(stream)

    def check_tab_error(self, space_count, tab_count, line=None, column=None):
        """Check TabError: Inconsistent use of tabs and spaces in indentation"""
        self._indent_has_space = True if space_count else self._indent_has_space
        self._indent_has_tab = True if tab_count else self._indent_has_tab
        if self._indent_has_space and self._indent_has_tab:
            import kclvm.compiler.parser.lark_parser as lark_parser

            kcl_error.report_exception(
                err_type=kcl_error.ErrType.TabError_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=lark_parser.filename, line_no=line, col_no=column
                    )
                ],
            )

    def handle_NL(self, token):
        """Do not edit it, inherit from base class 'Indenter'"""
        if self.paren_level > 0:
            return

        yield token

        indent_str = token.rsplit("\n", 1)[1]  # Tabs and spaces
        space_count, tab_count = indent_str.count(" "), indent_str.count("\t")
        indent = space_count + tab_count * self.tab_len
        self.check_tab_error(space_count, tab_count, token.end_line, token.end_column)

        if indent > self.indent_level[-1]:
            self.indent_level.append(indent)
            yield LarkToken.new_borrow_pos(self.INDENT_type, indent_str, token)
        else:
            while indent < self.indent_level[-1]:
                self.indent_level.pop()
                self.reset_indent_space_tab()
                lark_token = LarkToken.new_borrow_pos(
                    self.DEDENT_type, indent_str, token
                )
                yield lark_token

            if indent != self.indent_level[-1]:
                import kclvm.compiler.parser.lark_parser as lark_parser

                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.IndentationError_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=lark_parser.filename,
                            line_no=lark_token.end_line,
                            col_no=lark_token.end_column,
                        )
                    ],
                    arg_msg=kcl_error.INDENTATION_ERROR_MSG.format(str(indent)),
                )


_kcl_lark_parser: typing.Optional[Lark] = None


def GetKclLarkParser() -> Lark:
    global _kcl_lark_parser

    if _kcl_lark_parser is None:
        _kcl_lark_parser = Lark.open(
            "../../kcl/grammar/kcl.lark",
            parser="lalr",
            propagate_positions=True,
            postlex=KCLIndenter(),
            rel_to=__file__,
            cache=str(_CACHE_FILE),
        )

    return _kcl_lark_parser


def IsRuleType(node_type: str) -> bool:
    return node_type.islower()


def IsTokenType(node_type: str) -> bool:
    return node_type.isupper()


def GetNode(
    node: lark_pb.Tree, node_type: str, *more_node_type: str
) -> typing.Optional[lark_pb.Tree]:
    node_list = GetNodeList(node, node_type, *more_node_type, max_size=1)
    return node_list[0] if node_list else None


def GetNodeList(
    node: lark_pb.Tree,
    target_node_type: str,
    *more_target_node_type: str,
    max_size=0,
    recursively: bool = True
) -> typing.List[lark_pb.Tree]:
    node_type_list = [target_node_type, *more_target_node_type]

    if not node:
        return []

    if node.type in node_type_list:
        return [node]  # OK

    # try sub node
    node_list = []
    for n in node.children or []:
        if n.type in node_type_list:
            node_list.append(n)
            if 0 < max_size <= len(node_list):
                return node_list
            continue
        if recursively:
            node_list.extend(
                GetNodeList(
                    n, target_node_type, *more_target_node_type, max_size=max_size
                )
            )
        if 0 < max_size <= len(node_list):
            return node_list

    return node_list


def WalkTree(t: lark_pb.Tree, walk_fn):
    walk_fn(t)
    for n in t.children:
        WalkTree(n, walk_fn)


def ParseFile(filename: str, code: str, ignore_file_line: bool = False) -> lark_pb.Tree:
    if not code:
        with open(filename) as f:
            code = str(f.read())
    return ParseCode(code, ignore_file_line=ignore_file_line)


def ParseCode(src: str, ignore_file_line: bool = False) -> lark_pb.Tree:
    def _pb_build_Tree(_t: LarkTree) -> lark_pb.Tree:
        if isinstance(_t, LarkTree):
            rule_type = _t.data

            assert rule_type.islower()
            assert len(_t.children) >= 0

            # Empty file and return a empty lark tree node
            if rule_type == START_RULE and not _t.children:
                t = lark_pb.Tree(
                    type=rule_type,
                    token_value="",
                    children=[],
                )
            elif not ignore_file_line:
                t = lark_pb.Tree(
                    type=rule_type,
                    token_value="",  # rule, not token
                    children=[],
                    line=_t.meta.line,
                    column=_t.meta.column,
                    end_line=_t.meta.end_line,
                    end_column=_t.meta.end_column,
                )
            else:
                t = lark_pb.Tree(
                    type=rule_type, token_value="", children=[]  # rule, not token
                )

            for v in _t.children:
                t.children.append(_pb_build_Tree(v))

            return t

        if isinstance(_t, LarkToken):
            token_type = _t.type

            assert token_type.isupper()

            if not ignore_file_line:
                return lark_pb.Tree(
                    type=token_type,
                    token_value=_t.value,
                    children=[],
                    line=_t.line,
                    column=_t.column,
                    end_line=_t.end_line,
                    end_column=_t.end_column,
                )
            else:
                return lark_pb.Tree(type=token_type, token_value=_t.value, children=[])

        return lark_pb.Tree()

    # To prevent empty files and files that only contain line continuation symbols
    src += "\n"
    tree = GetKclLarkParser().parse(src)
    return _pb_build_Tree(tree)
