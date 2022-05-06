# Copyright 2021 The KCL Authors. All rights reserved.

import json
import sys
import traceback

from typing import List, Dict, Callable

import google.protobuf.json_format as json_format

import kclvm.kcl.error as kcl_error

from .gpyrpc_pb2 import (
    Error,
    Request,
    Response,
    ListMethod_Result,
    Ping_Args,
    Ping_Result,
)

# https://developers.google.com/protocol-buffers/docs/reference/google.protobuf
# https://developers.google.com/protocol-buffers/docs/reference/python-generated
# https://github.com/protocolbuffers/protobuf/tree/master/src/google/protobuf

_rpc_method_table: Dict[str, Callable[[Request], Response]] = {}


def _gpyrpcCallProxy(method: str, jsonpb_Request: str) -> str:
    assert method
    assert jsonpb_Request

    try:
        fn = _rpc_method_table[method]
        assert fn

        req = Request()
        json_format.Parse(jsonpb_Request, req)

        resp = Response()
        fn_result = fn(req)
        resp.CopyFrom(fn_result)

        jsonpb_Response: str = json_format.MessageToJson(
            resp, including_default_value_fields=True, preserving_proto_field_name=True
        )
        return jsonpb_Response

    except kcl_error.KCLException as err:
        errMessage = f"{err}"
        resp = Response(err=Error(message=errMessage))
        return json_format.MessageToJson(
            resp, including_default_value_fields=True, preserving_proto_field_name=True
        )
    except OSError as err:
        errMessage = f"OSError: {err}"
        resp = Response(err=Error(message=errMessage))
        return json_format.MessageToJson(
            resp, including_default_value_fields=True, preserving_proto_field_name=True
        )
    except AssertionError as err:
        ex_type, ex_val, ex_stack = sys.exc_info()
        tb_info = traceback.extract_tb(ex_stack)[-1]

        filename = tb_info.filename
        line = tb_info.lineno

        errMessage = f"AssertionError: {err}"
        resp = Response(err=Error(message=errMessage, filename=filename, line=line))
        return json_format.MessageToJson(
            resp, including_default_value_fields=True, preserving_proto_field_name=True
        )
    except Exception as err:
        errMessage = f"Exception: Internal Error! Please report a bug to us: method={method}, err={err}, stack trace={traceback.format_exc()}"
        resp = Response(err=Error(message=errMessage))
        return json_format.MessageToJson(
            resp, including_default_value_fields=True, preserving_proto_field_name=True
        )


def gpyrpcCallProxy(method: str, jsonRequest: str) -> str:
    if not method:
        errMessage = "method is empty"
        resp = Response(err=Error(message=errMessage))
        return json_format.MessageToJson(
            resp, including_default_value_fields=True, preserving_proto_field_name=True
        )

    if method not in _rpc_method_table:
        errMessage = f"method '{method}' not found"
        resp = Response(err=Error(message=errMessage))
        return json_format.MessageToJson(
            resp, including_default_value_fields=True, preserving_proto_field_name=True
        )
        resp: Response = {"error": f"method '{method}' not found"}
        return str(json.dumps(resp))

    return _gpyrpcCallProxy(method, jsonRequest)


def RegisterMethod(method: str, fn: Callable[[Request], Response]):
    assert method
    assert fn

    assert method not in _rpc_method_table
    _rpc_method_table[method] = fn


# args: gpyrpc.Ping_Args
# result: gpyrpc.Ping_Result
def _gpyrpcPing(req: Request) -> Response:
    args = Ping_Args()
    req.args.Unpack(args)

    resp = Response()
    resp.result.Pack(Ping_Result(value=args.value))
    return resp


# args: gpyrpc.ListMethod_Args
# result: gpyrpc.ListMethod_Result
def _gpyrpcListMethod(req: Request) -> Response:
    keys: List[str] = list(sorted(_rpc_method_table))

    resp = Response()
    resp.result.Pack(ListMethod_Result(method_name_list=keys))

    return resp


RegisterMethod("gpyrpc.BuiltinService.Ping", _gpyrpcPing)
RegisterMethod("gpyrpc.BuiltinService.ListMethod", _gpyrpcListMethod)
