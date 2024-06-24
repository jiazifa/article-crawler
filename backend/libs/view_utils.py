# 这里是跟 view 相关的工具函数，比如返回 json 数据的函数，返回错误信息的函数等等

from typing import Any, Dict, Generic, Optional, TypeVar

from flask import Response, jsonify
from flask_restx import Model, fields
from libs.helper import get_current_time


def to_camel(string: str) -> str:
    parts = string.split("_")
    return parts[0] + "".join(part.title() for part in parts[1:])


T = TypeVar("T")


class ResponseCtx:
    code: int = 200
    msg: str = ""
    server_at: int = 0

    def to_dict(self) -> Dict[str, Any]:
        return {"code": self.code, "msg": self.msg, "server_at": self.server_at}

    def __init__(
        self, code: int = 200, msg: str = "", server_at: Optional[int] = None
    ) -> None:
        self.code = code
        self.msg = msg
        self.server_at = server_at or get_current_time()


def get_default_response_message() -> ResponseCtx:
    return ResponseCtx(code=200, msg="", server_at=get_current_time())


class ResponseModel(Generic[T]):
    data: T
    context: ResponseCtx

    def __init__(self, data: T, context: Optional[ResponseCtx] = None) -> None:
        self.data = data
        self.context = context or get_default_response_message()

    @classmethod
    def from_error(cls, msg: str, code: int = 400) -> "ResponseModel":
        return cls(data=None, context=ResponseCtx(code=code, msg=msg))


class PageRequest:
    page: int = 1
    size: int = 10


def make_response(
    body: ResponseModel[T],
    status_code: int = 200,
    header: Optional[Dict[str, str]] = None,
) -> Response:
    response: Response

    result: Dict[str, Any] = {}
    if body.context:
        result["context"] = body.context.to_dict()
    if body.data:
        result["data"] = body.data

    response = jsonify(result)
    if header:
        for key, value in header.items():
            response.headers[key] = value

    if status_code != 200:
        response.status_code = status_code

    return response


# we define the response model here
response_context_model = Model(
    "ResponseContextModel",
    {
        "code": fields.Integer(
            readonly=True, required=False, description="Response code"
        ),
        "msg": fields.String(readonly=True, required=False, description="Message"),
        "server_at": fields.Integer(
            readonly=True, required=False, description="Server time"
        ),
    },
)
response_schema = Model(
    "ResponseModel",
    {
        "data": fields.Raw(required=False, readonly=True, description="Response data"),
        "context": fields.Nested(response_context_model),
    },
)
