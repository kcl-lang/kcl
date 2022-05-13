# Copyright 2021 The KCL Authors. All rights reserved.

import abc
import sys
import traceback
import typing

from dataclasses import dataclass

import kclvm.kcl.error as kcl_error
import kclvm.internal.gpyrpc.varint as varint

from google.protobuf import message as _message
import google.protobuf.json_format as _json_format

from .protorpc_wire_pb2 import (
    MAX_REQUEST_HEADER_LEN,
    RequestHeader,
    ResponseHeader,
)


@dataclass
class Status:
    error: str = ""


class Channel(object):
    def __init__(self, *, stdin=sys.stdin, stdout=sys.stdout):
        self.stdin = stdin
        self.stdout = stdout
        self.next_id = 1

    def get_next_id(self) -> int:
        next_id = self.next_id
        self.next_id = self.next_id + 1
        return next_id

    def send_frame(self, data: bytes):
        self.stdout.buffer.write(varint.encode(len(data)))
        self.stdout.buffer.write(data)
        self.stdout.flush()

    def recv_frame(self, _max_size: int = 0) -> bytes:
        size = varint.decode_stream(self.stdin.buffer)

        if size > _max_size > 0:
            raise Exception(f"protorpc: varint overflows maxSize({_max_size})")

        data = self.stdin.buffer.read(size)
        return data

    def write_request(self, method: str, req: _message.Message):
        body = req.SerializeToString()

        hdr = RequestHeader(
            id=self.get_next_id(), method=method, raw_request_len=len(body)
        )
        self.send_frame(hdr.SerializeToString())
        self.send_frame(body)

    def read_request_header(self) -> RequestHeader:
        data = self.recv_frame(MAX_REQUEST_HEADER_LEN)
        hdr = RequestHeader()
        hdr.ParseFromString(data)

        if hdr.snappy_compressed_request_len != 0:
            raise Exception("py: unsupport snappy compressed request")
        if hdr.checksum != 0:
            raise Exception("py: unsupport checksum request")

        return hdr

    def read_request_body(self, header: RequestHeader, body: _message.Message):
        data = self.recv_frame(
            max(header.raw_request_len, header.snappy_compressed_request_len)
        )
        body.ParseFromString(data)

    def write_response(self, id_: int, error: str, response: _message.Message):
        body = response.SerializeToString()
        hdr = ResponseHeader(id=id_, error=error, raw_response_len=len(body))
        self.send_frame(hdr.SerializeToString())
        self.send_frame(body)

    def read_response_header(self) -> ResponseHeader:
        data = self.recv_frame(0)
        hdr = ResponseHeader()
        hdr.ParseFromString(data)

        if hdr.snappy_compressed_response_len != 0:
            raise Exception("py: unsupport snappy compressed response")
        if hdr.checksum != 0:
            raise Exception("py: unsupport checksum response")

        return hdr

    def read_response_body(self, header: ResponseHeader, body: _message.Message):
        data = self.recv_frame(
            max(header.raw_response_len, header.snappy_compressed_response_len)
        )
        body.ParseFromString(data)

    def call_method(
        self, method: str, req: _message.Message, resp: _message.Message
    ) -> Status:
        self.write_request(method, req)
        resp_hdr = self.read_response_header()
        self.read_response_body(resp_hdr, resp)
        return Status(error=resp_hdr.error)


class ServiceMeta(metaclass=abc.ABCMeta):
    @abc.abstractmethod
    def get_service_name(self) -> str:
        pass

    @abc.abstractmethod
    def get_method_list(self) -> typing.List[str]:
        pass

    @abc.abstractmethod
    def create_method_req_message(self, method: str) -> _message.Message:
        pass

    @abc.abstractmethod
    def create_method_resp_message(self, method: str) -> _message.Message:
        pass

    @abc.abstractmethod
    def get_service_instance(self) -> _message.Message:
        pass

    @abc.abstractmethod
    def call_method(self, method: str, req: _message.Message) -> _message.Message:
        pass


class Server:
    def __init__(self):
        self.srv_table: typing.Dict[str, ServiceMeta] = {}
        self.chan: typing.Optional[Channel] = None

    def register_service(self, srv: ServiceMeta):
        self.srv_table[srv.get_service_name()] = srv

    def get_service_name_list(self) -> typing.List[str]:
        name_list: typing.List[str] = []
        for s in self.srv_table.keys():
            name_list.append(s)
        name_list.sort()
        return name_list

    def get_method_name_list(self) -> typing.List[str]:
        name_list: typing.List[str] = []
        for srv in self.srv_table.values():
            srv_name = srv.get_service_name()
            for method_name in srv.get_method_list():
                name_list.append(f"{srv_name}.{method_name}")
        name_list.sort()
        return name_list

    def run(self, *, stdin=sys.stdin, stdout=sys.stdout):
        self.chan = Channel(stdin=stdin, stdout=stdout)
        while True:
            self._accept_one_call()

    def run_once(self, *, stdin=sys.stdin, stdout=sys.stdout):
        self.chan = Channel(stdin=stdin, stdout=stdout)
        self._accept_one_call()

    def call_method(
        self, method_path: str, req_body: bytes, *, encoding: str = "json"
    ) -> dict:
        if encoding not in ["json", "protobuf"]:
            raise Exception(f"encoding '{encoding}' not support")

        service_name = method_path[: method_path.rfind(".")]
        method_name = method_path[method_path.rfind(".") + 1 :]

        if service_name not in self.srv_table:
            raise Exception(f"service '{service_name}' not found")

        service = self.srv_table[service_name]

        req = service.create_method_req_message(method_name)

        if encoding == "json":
            _json_format.Parse(req_body, req)
        else:
            req.ParseFromString(req_body)

        try:
            resp = service.call_method(method_name, req)

        except kcl_error.KCLException as err:
            raise err

        except OSError as err:
            err_message = f"OSError: {err}"
            raise Exception(err_message)

        except AssertionError as err:
            err_message = f"AssertionError: {err}"
            raise Exception(err_message)

        except Exception as err:
            err_message = f"Exception: Internal Error! Please report a bug to us: method={method_name}, err={err}, stack trace={traceback.format_exc()}"
            raise Exception(err_message)

        # return response
        # https://googleapis.dev/python/protobuf/latest/google/protobuf/json_format.html
        return _json_format.MessageToDict(
            resp,
            including_default_value_fields=True,
            preserving_proto_field_name=True,
        )

    def _accept_one_call(self):
        hdr = self._read_req_header()

        service_name = hdr.method[: hdr.method.rfind(".")]
        method_name = hdr.method[hdr.method.rfind(".") + 1 :]

        if service_name not in self.srv_table:
            raise Exception(f"service '{service_name}' not found")

        service = self.srv_table[service_name]

        req = self._read_req(service, hdr)

        try:
            resp = service.call_method(method_name, req)
            self._write_resp(hdr.id, "", resp)
            return

        except kcl_error.KCLException as err:
            resp = service.create_method_resp_message(method_name)

            err_message = f"{err}"
            self._write_resp(hdr.id, err_message, resp)
            return

        except OSError as err:
            resp = service.create_method_resp_message(method_name)

            err_message = f"OSError: {err}"
            self._write_resp(hdr.id, err_message, resp)
            return

        except AssertionError as err:
            resp = service.create_method_resp_message(method_name)

            err_message = f"AssertionError: {err}"
            self._write_resp(hdr.id, err_message, resp)
            return

        except Exception as err:
            resp = service.create_method_resp_message(method_name)

            err_message = f"Exception: Internal Error! Please report a bug to us: method={method_name}, err={err}, stack trace={traceback.format_exc()}"
            self._write_resp(hdr.id, err_message, resp)
            return

    def _read_req_header(self) -> RequestHeader:
        return self.chan.read_request_header()

    def _read_req(self, service: ServiceMeta, hdr: RequestHeader) -> _message.Message:
        req = service.create_method_req_message(hdr.method)
        self.chan.read_request_body(hdr, req)
        return req

    def _write_resp(self, id_: int, error: str, resp: _message.Message):
        self.chan.write_response(id_, error, resp)


class Client:
    def __init__(self, chan: Channel = None):
        self.chan: Channel = chan

    def call_method(
        self, method: str, req: _message.Message, resp: _message.Message
    ) -> Status:
        return self.chan.call_method(method, req, resp)
