import abc
import sys
import typing

import varint

from google.protobuf import message as _message

from .protorpc_wire_pb2 import (
    RequestHeader,
    ResponseHeader,
)


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
        data = self.recv_frame()
        hdr = RequestHeader()
        hdr.ParseFromString(data)
        return hdr

    def read_request_body(self, _header: RequestHeader, body: _message.Message):
        data = self.recv_frame()
        body.ParseFromString(data)

    def write_response(self, id_: int, error: str, response: _message.Message):
        if not error:
            body = response.SerializeToString()
        else:
            body = ""

        hdr = ResponseHeader(id=id_, error=error, raw_response_len=len(body))
        self.send_frame(hdr.SerializeToString())

        if not error:
            self.send_frame(body)

    def read_response_header(self) -> ResponseHeader:
        data = self.recv_frame()
        hdr = ResponseHeader()
        hdr.ParseFromString(data)
        return hdr

    def read_response_body(self, header: ResponseHeader, body: _message.Message):
        if header.error:
            raise header.error
        data = self.recv_frame()
        body.ParseFromString(data)

    def call_method(self, method: str, req: _message.Message, resp: _message.Message):
        self.write_request(method, req)
        resp_hdr = self.read_response_header()
        self.read_response_body(resp_hdr, resp)


class ServiceMeta(metaclass=abc.ABCMeta):
    def __init__(self, instance: _message.Message):
        self._instance = instance

    def get_service_instance(self) -> _message.Message:
        return self._instance

    def call_method(self, method: str, req: _message.Message) -> _message.Message:
        return getattr(self.get_service_instance(), method[method.rfind(".") + 1:])(
            req
        )

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
        for s in self.srv_table.values():
            name_list.extend(s.get_method_list())
        name_list.sort()
        return name_list

    def run(self, *, stdin=sys.stdin, stdout=sys.stdout):
        self.chan = Channel(stdin=stdin, stdout=stdout)
        while True:
            self._accept_one_call()

    def _accept_one_call(self):
        hdr = self._read_req_header()

        service_name = hdr.method[: hdr.method.rfind(".")]
        method_name = hdr.method[hdr.method.rfind(".") + 1:]
        service = self.srv_table[service_name]

        req = self._read_req(service, hdr)
        resp = service.call_method(method_name, req)

        self._write_resp(hdr.id, "", resp)

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

    def call_method(self, method: str, req: _message.Message, resp: _message.Message):
        self.chan.call_method(method, req, resp)
