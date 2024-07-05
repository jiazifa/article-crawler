from typing import Generic, Optional, TypeVar

from pydantic import BaseModel, Field

from app.utils import get_current_time

SEPERATE_OP = "data: "


def to_camel(string: str) -> str:
    parts = string.split("_")
    return parts[0] + "".join(part.title() for part in parts[1:])


T = TypeVar("T")


class ResponseContext(BaseModel):
    status: int = Field(default=200)
    message: str = Field(default="")
    server_at: int = Field(default_factory=get_current_time)


def get_default_response_message() -> ResponseContext:
    return ResponseContext(status=200, message="", server_at=get_current_time())


class Response(BaseModel, Generic[T]):
    data: Optional[T] = Field(...)
    context: ResponseContext = Field(default_factory=get_default_response_message)

    @classmethod
    def from_error(cls, message: str, status: int = 500) -> "Response":
        return cls(data=None, context=ResponseContext(status=status, message=message))
