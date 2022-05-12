# Copyright 2021 The KCL Authors. All rights reserved.

import json
import sys
import socket
import subprocess
import typing
import time
import pathlib

from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, HTTPServer

import kclvm.kcl.ast as ast
import kclvm.kcl.error as kcl_error
import kclvm.kcl.types as types
import kclvm.config
import kclvm.compiler.extension.plugin as plugin
import kclvm.compiler.parser.lark_parser as lark_parser
import kclvm.compiler.parser.parser as parser
import kclvm.program.exec as kclvm_exec
import kclvm.vm.planner as planner
import kclvm.internal.gpyrpc.gpyrpc_pb2 as pb2
import kclvm.internal.gpyrpc.gpyrpc_pb_protorpc as pbrpc
import kclvm.internal.gpyrpc.protorpc as protorpc
from kclvm.tools.format import kcl_fmt_source, kcl_fmt
from kclvm.tools.lint.lint import kcl_lint
from kclvm.tools.query import override_file
from kclvm.tools.validation import validate_code_with_attr_data
from kclvm.tools.langserver import grpc_wrapper
from kclvm.tools.printer import SchemaRuleCodeSnippet, splice_schema_with_rule
from kclvm.tools.list_attribute.schema import get_schema_type_from_code

import kclvm.kcl.error.kcl_err_template as kcl_err_template

import google.protobuf.json_format as _json_format

USAGE = """\
usage: kclvm -m kclvm.program.rpc-server             # run server on stdin/stdout
       kclvm -m kclvm.program.rpc-server -http=:2021 # run fast http server on port
       kclvm -m kclvm.program.rpc-server -h
"""


@dataclass
class CmdFlags:
    help: bool = False

    http_addr: str = ""
    http_port: int = 0

    use_http_server: bool = False


def parse_flags(args: typing.List[str]) -> CmdFlags:
    m = CmdFlags()
    for s in args:
        if s == "-h" or s == "-help":
            m.help = True
            continue

        if s.startswith("-http="):
            value = s[len("-http=") :]
            m.http_addr = str(value[: value.find(":")])
            m.http_port = int(value[value.find(":") + 1 :])
            continue

        if s.startswith("-http.server="):
            value = s[len("-http.server=") :]
            m.http_addr = str(value[: value.find(":")])
            m.http_port = int(value[value.find(":") + 1 :])
            m.use_http_server = True
            continue

    if m.http_port and not m.http_addr:
        m.http_addr = "localhost"

    return m


class ASTTransformer(ast.TreeTransformer):
    def walk_NameConstantLit(self, t: ast.NameConstantLit):
        t.value = str(t.value)
        return t


# KclvmService implementation
class KclvmServiceImpl(pbrpc.KclvmService):
    def Ping(self, args: pb2.Ping_Args) -> pb2.Ping_Result:
        return pb2.Ping_Result(value=args.value)

    def ParseFile_LarkTree(
        self, args: pb2.ParseFile_LarkTree_Args
    ) -> pb2.ParseFile_LarkTree_Result:
        result = pb2.ParseFile_LarkTree_Result()

        if args.source_code:
            t = lark_parser.ParseFile(
                args.filename, args.source_code, ignore_file_line=args.ignore_file_line
            )
            result.lark_tree_json = json.dumps(t, default=lambda obj: obj.__dict__)

        return result

    def ParseFile_AST(self, args: pb2.ParseFile_AST_Args) -> pb2.ParseFile_AST_Result:
        t = parser.ParseFile(args.filename, args.source_code)
        ASTTransformer().walk(t)
        result = pb2.ParseFile_AST_Result()
        result.ast_json = t.to_json()
        return result

    def ParseProgram_AST(
        self, args: pb2.ParseProgram_AST_Args
    ) -> pb2.ParseProgram_AST_Result:
        program = parser.LoadProgram(*list(args.k_filename_list))
        for pkgpath in program.pkgs:
            for i, module in enumerate(program.pkgs[pkgpath]):
                program.pkgs[pkgpath][i] = ASTTransformer().walk(module)
        result = pb2.ParseProgram_AST_Result()
        result.ast_json = program.to_json()
        return result

    def ExecProgram(self, args: pb2.ExecProgram_Args) -> pb2.ExecProgram_Result:
        cmd_args: typing.List[ast.CmdArgSpec] = []
        cmd_overrides: typing.List[ast.CmdOverrideSpec] = []

        # kcl -D name=value main.k
        for x in args.args:
            cmd_args.append(ast.CmdArgSpec(name=x.name, value=x.value))

        # kcl main.k -O pkgpath:path.to.field=field_value
        for x in args.overrides:
            cmd_overrides.append(
                ast.CmdOverrideSpec(
                    pkgpath=x.pkgpath or "__main__",
                    field_path=x.field_path,
                    field_value=x.field_value,
                    action=ast.OverrideAction(x.action)
                    if x.action
                    else ast.OverrideAction.CREATE_OR_UPDATE,
                )
            )

        work_dir: str = args.work_dir
        k_filename_list: typing.List[str] = list(args.k_filename_list)
        k_filename_list = [
            str(
                pathlib.Path(work_dir or kclvm.config.current_path or "").joinpath(file)
            )
            if file.startswith(".")
            else file
            for file in k_filename_list or []
        ]
        k_code_list: typing.List[str] = list(args.k_code_list)
        disable_yaml_result: bool = args.disable_yaml_result
        print_override_ast: bool = args.print_override_ast

        # -r --strict-range-check
        strict_range_check: bool = args.strict_range_check
        # -n --disable-none
        disable_none: bool = args.disable_none
        # -v --verbose
        verbose: int = args.verbose
        # -d --debug
        debug: int = args.debug

        sort_keys: bool = args.sort_keys
        include_schema_type_path: bool = args.include_schema_type_path

        start_time = time.time()
        kcl_result = kclvm_exec.Run(
            k_filename_list,
            work_dir=work_dir,
            k_code_list=k_code_list,
            cmd_args=cmd_args,
            cmd_overrides=cmd_overrides,
            print_override_ast=print_override_ast,
            strict_range_check=strict_range_check,
            disable_none=disable_none,
            verbose=verbose,
            debug=debug,
        )
        end_time = time.time()

        result = pb2.ExecProgram_Result()

        result.escaped_time = f"{end_time-start_time}"

        # json
        output_json = planner.JSONPlanner(
            sort_keys=sort_keys, include_schema_type_path=include_schema_type_path
        ).plan(
            kcl_result.filter_by_path_selector(
                to_kcl=not kclvm.config.is_target_native
            ),
            to_py=not kclvm.config.is_target_native,
        )
        result.json_result = output_json

        # yaml
        if not disable_yaml_result:
            output_yaml = planner.YAMLPlanner(
                sort_keys=sort_keys, include_schema_type_path=include_schema_type_path
            ).plan(
                kcl_result.filter_by_path_selector(
                    to_kcl=not kclvm.config.is_target_native
                ),
                to_py=not kclvm.config.is_target_native,
            )
            result.yaml_result = output_yaml

        return result

    def ResetPlugin(self, args: pb2.ResetPlugin_Args) -> pb2.ResetPlugin_Result:
        plugin.reset_plugin(args.plugin_root)
        result = pb2.ResetPlugin_Result()
        return result

    def FormatCode(self, args: pb2.FormatCode_Args) -> pb2.FormatCode_Result:
        formatted, _ = kcl_fmt_source(args.source)
        return pb2.FormatCode_Result(formatted=formatted.encode("utf-8"))

    def FormatPath(self, args: pb2.FormatPath_Args) -> pb2.FormatPath_Result:
        path = args.path
        recursively = False
        if path.endswith("..."):
            recursively = True
            path = path[: len(path) - 3]
            if path == "" or path is None:
                path = "."
        changed_paths = kcl_fmt(path, recursively=recursively)
        return pb2.FormatPath_Result(changedPaths=changed_paths)

    def LintPath(self, args: pb2.LintPath_Args) -> pb2.LintPath_Result:
        path = args.path

        results: typing.List[str] = []
        for lintMessage in kcl_lint(path):
            results.append(f"{lintMessage.msg}")

        return pb2.LintPath_Result(results=results)

    def OverrideFile(self, args: pb2.OverrideFile_Args) -> pb2.OverrideFile_Result:
        result = override_file(args.file, args.specs, args.import_paths)
        return pb2.OverrideFile_Result(result=result)

    def EvalCode(self, args: pb2.EvalCode_Args) -> pb2.EvalCode_Result:
        import tempfile
        import os

        work_dir = tempfile.mkdtemp()

        with open(f"{work_dir}/kcl.mod", "w") as f:
            pass
        with open(f"{work_dir}/main.k", "w") as f:
            f.write(args.code)

        kcl_result = kclvm_exec.Run(
            ["main.k"], work_dir=work_dir, k_code_list=[args.code]
        )
        output_json = planner.JSONPlanner().plan(
            kcl_result.filter_by_path_selector(), only_first=True
        )

        os.remove(f"{work_dir}/kcl.mod")
        os.remove(f"{work_dir}/main.k")

        result = pb2.EvalCode_Result(json_result=output_json)
        return result

    def ResolveCode(self, args: pb2.ResolveCode_Args) -> pb2.ResolveCode_Result:
        import tempfile
        import os

        work_dir = tempfile.mkdtemp()

        with open(f"{work_dir}/kcl.mod", "w") as f:
            pass
        with open(f"{work_dir}/main.k", "w") as f:
            f.write(args.code)

        ast_prog = parser.LoadProgram(
            *["main.k"],
            work_dir=work_dir,
            k_code_list=[args.code],
        )
        types.ResolveProgram(ast_prog)

        os.remove(f"{work_dir}/kcl.mod")
        os.remove(f"{work_dir}/main.k")

        result = pb2.ResolveCode_Result(success=True)
        return result

    def GetSchemaType(self, args: pb2.GetSchemaType_Args) -> pb2.GetSchemaType_Result:
        schema_type_list = get_schema_type_from_code(
            args.file, args.code, args.schema_name
        )
        return pb2.GetSchemaType_Result(schema_type_list=schema_type_list)

    def ValidateCode(self, args: pb2.ValidateCode_Args) -> pb2.ValidateCode_Result:
        data: str = args.data
        code: str = args.code
        schema: typing.Optional[str] = args.schema or None
        format_: str = args.format or "JSON"

        success = validate_code_with_attr_data(data, code, schema, format_)
        return pb2.ValidateCode_Result(success=success)

    def SpliceCode(self, args: pb2.SpliceCode_Args):
        code_snippets_pb = args.codeSnippets
        code_snippets = [
            SchemaRuleCodeSnippet(
                schema=code_snippet.schema,
                rule=code_snippet.rule,
            )
            for code_snippet in code_snippets_pb
        ]
        splice_code = splice_schema_with_rule(code_snippets)
        return pb2.SpliceCode_Result(spliceCode=splice_code)

    def Complete(self, args: pb2.Complete_Args) -> pb2.Complete_Result:
        pos: pb2.Position = args.pos
        name: str = args.name
        code: str = args.code

        complete_items = grpc_wrapper.complete_wrapper(pos=pos, name=name, code=code)
        return pb2.Complete_Result(completeItems=complete_items)

    def GoToDef(self, args: pb2.GoToDef_Args) -> pb2.GoToDef_Result:
        pos: pb2.Position = args.pos
        code: str = args.code

        locations = grpc_wrapper.go_to_def_wrapper(pos=pos, code=code)
        return pb2.GoToDef_Result(locations=locations)

    def DocumentSymbol(
        self, args: pb2.DocumentSymbol_Args
    ) -> pb2.DocumentSymbol_Result:
        file: str = args.file
        code: str = args.code
        symbol = grpc_wrapper.document_symbol_wrapper(file=file, code=code)
        return pb2.DocumentSymbol_Result(symbol=symbol)

    def Hover(self, args: pb2.Hover_Args) -> pb2.Hover_Result:
        pos: pb2.Position = args.pos
        code: str = args.code
        hover_result = grpc_wrapper.hover_wrapper(pos=pos, code=code)
        return pb2.Hover_Result(hoverResult=hover_result)

    _kcl_go_exe: str = ""

    def ListDepFiles(self, args: pb2.ListDepFiles_Args) -> pb2.ListDepFiles_Result:
        if not self._kcl_go_exe:
            import os

            if os.name == "nt":
                _executable_root = os.path.dirname(sys.executable)
                self._kcl_go_exe = f"{_executable_root}/kcl-go.exe"
            else:
                _executable_root = os.path.dirname(os.path.dirname(sys.executable))
                self._kcl_go_exe = f"{_executable_root}/bin/kcl-go"

        # kcl-go list-app -use-fast-parser=<bool> -show-abs=<bool> -show-index=false <work_dir>
        args = [
            self._kcl_go_exe,
            "list-app",
            f"-use-fast-parser={args.use_fast_parser}",
            f"-include-all={args.include_all}",
            f"-show-abs={args.use_abs_path}",
            "-show-index=false",
            args.work_dir,
        ]

        proc = subprocess.run(args, capture_output=True, text=True)
        stdout = str(proc.stdout or "").strip()
        stderr = str(proc.stderr or "").strip()

        if proc.returncode != 0:
            if stdout and stderr:
                raise Exception(f"stdout: {stdout}, stderr: {stderr}")
            else:
                raise Exception(stdout if stdout else stderr)

        pkgroot: str = ""
        pkgpath: str = ""
        files: typing.List[str] = []

        for s in stdout.splitlines():
            if s.startswith("pkgroot:"):
                pkgroot = s[len("pkgroot:") :]
                pkgroot = s.strip()
            elif s.startswith("pkgpath:"):
                pkgpath = s[len("pkgpath:") :]
                pkgpath = s.strip()
            else:
                s = s.strip()
                if s:
                    files.append(s)

        return pb2.ListDepFiles_Result(pkgroot=pkgroot, pkgpath=pkgpath, files=files)

    def LoadSettingsFiles(
        self, args: pb2.LoadSettingsFiles_Args
    ) -> pb2.LoadSettingsFiles_Result:
        return kclvm.config.load_settings_files(args.work_dir, args.files)


def _makeRpcServer():
    rpc_server = protorpc.Server()

    # BuiltinService implementation (depends on rpc_server)
    class BuiltinServiceImpl(pbrpc.BuiltinService):
        def Ping(self, args: pb2.Ping_Args) -> pb2.Ping_Result:
            return pb2.Ping_Result(value=args.value)

        def ListMethod(self, args: pb2.ListMethod_Args) -> pb2.ListMethod_Result:
            return pb2.ListMethod_Result(
                method_name_list=rpc_server.get_method_name_list()
            )

    # Metaclass for packaging services
    builtin_service = pbrpc.BuiltinService_Meta(
        typing.cast(pbrpc.BuiltinService, BuiltinServiceImpl())
    )
    kclvm_service = pbrpc.KclvmService_Meta(
        typing.cast(pbrpc.KclvmService, KclvmServiceImpl())
    )

    # Register service metaclass
    rpc_server.register_service(builtin_service)
    rpc_server.register_service(kclvm_service)

    return rpc_server


def runStdioProtorpcServer():
    """Start protorpc service based on stdin/stdout"""

    # Redirect stdout: raw_stdout raw_stdout will be used for protorpc communication
    raw_stdout = sys.stdout
    sys.stdout = sys.stderr

    # Start the service based on raw_stdout (blocking)
    rpc_server = _makeRpcServer()
    rpc_server.run(stdout=raw_stdout)


def runHttpServer(*, addr: str = "", port: int = 2021):

    rpc_server = _makeRpcServer()

    class MyHTTPServer(HTTPServer):
        def server_bind(self):
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
            self.socket.bind(self.server_address)

    class httpHandler(BaseHTTPRequestHandler):
        def do_GET(self):

            # http://localhost:2021/api:protorpc/BuiltinService.Ping
            # http://localhost:2021/api:protorpc/BuiltinService.ListMethod

            # Build protorpc call
            if self.path.startswith("/api:protorpc/"):
                self.send_response(200)
                self.send_header("Content-type", "application/json")
                self.end_headers()

                from urllib.parse import urlparse

                u = urlparse(self.path)
                method = u.path[len("/api:protorpc/") :]

                resp = {}
                try:
                    result = rpc_server.call_method(method, b"{}", encoding="json")
                    resp["result"] = result
                    resp["error"] = ""
                except kcl_error.KCLException as err:
                    resp["result"] = None
                    resp["error"] = f"{err}"
                    resp["kcl_err"] = kcl_err_to_response_dict(err)
                except Exception as err:
                    resp["result"] = None
                    resp["error"] = f"{err}"
                self.wfile.write(bytes(json.dumps(resp), "utf8"))
            else:
                self.send_response(200)
                self.send_header("Content-type", "text/html")
                self.end_headers()

                message = f"Hello, World! Here is a GET response: path={self.path}"
                self.wfile.write(bytes(message, "utf8"))

        def do_POST(self):
            self.send_response(200)
            self.send_header("Content-type", "application/octet-stream")
            self.end_headers()

            content_len = int(self.headers.get("Content-Length"))
            req_body = self.rfile.read(content_len)

            # Build protorpc call
            if self.path.startswith("/api:protorpc/"):
                method = self.path[len("/api:protorpc/") :]
                resp = {}
                try:
                    result = rpc_server.call_method(method, req_body, encoding="json")
                    resp["result"] = result
                    resp["error"] = ""
                except kcl_error.KCLException as err:
                    resp["result"] = None
                    resp["error"] = f"{err}"
                    resp["kcl_err"] = kcl_err_to_response_dict(err)
                except Exception as err:
                    resp["result"] = None
                    resp["error"] = f"{err}"
                self.wfile.write(bytes(json.dumps(resp), "utf8"))
            else:
                message = f"Hello, World! Here is a POST response: path={self.path}"
                self.wfile.write(bytes(message, "utf8"))

    print(f"run http server on http://{addr}:{port} ...")

    with MyHTTPServer(("", port), httpHandler) as server:
        server.serve_forever()


def runFastApiServer(*, addr: str = "", port: int = 2021):
    import uvicorn

    app = create_app()
    uvicorn.run(app, host=addr, port=port)


def kcl_err_to_response_dict(err: kcl_error.KCLException) -> pb2.KclError:
    return _json_format.MessageToDict(
        pb2.KclError(
            ewcode=f"{err.ewcode}",
            name=f"{err.name}",
            msg=f"{err.arg_msg}",
            error_infos=[
                pb2.KclErrorInfo(
                    err_level=f"{err_info.err_level}",
                    arg_msg=f"{err_info.arg_msg}",
                    filename=f"{err_info.filename}",
                    src_code=f"{kcl_err_template.get_src_code(err_info)}",
                    line_no=f"{err_info.line_no}",
                    col_no=f"{err_info.col_no}",
                )
                for err_info in err.err_info_stack
            ],
        ),
        including_default_value_fields=True,
        preserving_proto_field_name=True,
    )


# ./gunicorn "kclvm.program.rpc-server:create_app()" -w 4 -k uvicorn.workers.UvicornWorker -b :2021
def create_app():
    import fastapi

    app = fastapi.FastAPI()
    rpc_server = _makeRpcServer()

    @app.get("/")
    async def index():
        return "KCL Rest Server"

    @app.get("/api:protorpc/{method}")
    async def on_rest_api_get(method: str, request: fastapi.Request):
        resp = {}
        try:
            result = rpc_server.call_method(method, b"{}", encoding="json")
            resp["result"] = result
            resp["error"] = ""
        except kcl_error.KCLException as err:
            resp["result"] = None
            resp["error"] = f"{err}"
            resp["kcl_err"] = kcl_err_to_response_dict(err)
        except Exception as err:
            resp["result"] = None
            resp["error"] = f"{err}"

        return resp

    @app.post("/api:protorpc/{method}")
    async def on_rest_api_post(method: str, request: bytes = fastapi.Body(...)):
        resp = {}
        try:
            result = rpc_server.call_method(method, request, encoding="json")
            resp["result"] = result
            resp["error"] = ""
        except kcl_error.KCLException as err:
            resp["result"] = None
            resp["error"] = f"{err}"
            resp["kcl_err"] = kcl_err_to_response_dict(err)
        except Exception as err:
            resp["result"] = None
            resp["error"] = f"{err}"

        return resp

    return app


def main():
    flags = parse_flags(sys.argv[1:])

    if flags.help:
        print(USAGE)
        sys.exit(0)

    from kclvm.compiler.parser.lark_parser import GetKclLarkParser

    GetKclLarkParser()

    if flags.http_addr or flags.http_port:
        print(f"flags: {flags}")
        if not flags.use_http_server:
            runFastApiServer(addr=flags.http_addr, port=flags.http_port)
        else:
            runHttpServer(addr=flags.http_addr, port=flags.http_port)
        sys.exit(0)
    else:
        runStdioProtorpcServer()
        sys.exit(0)


if __name__ == "__main__":
    main()
