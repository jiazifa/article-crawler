from datetime import datetime, timedelta, timezone
from typing import Optional, Union

from jose import jwt
from pydantic import BaseModel

from app.settings import settings


class AuthClaims(BaseModel):
    sub: str  # "subject / user id"
    iat: Optional[datetime] = None  # "issue at 签发时间"

    nbf: Optional[datetime] = None  # "not before"
    exp: Optional[datetime] = None  # "expire at"


# 创建 token
def create_access_token(
    claims: AuthClaims, expires_delta: Union[timedelta, None] = None
):
    to_encode = claims.model_dump()
    if expires_delta:
        expire = datetime.now(timezone.utc) + expires_delta
    else:
        expire = datetime.now(timezone.utc) + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(
        to_encode, settings.SECRET_KEY, algorithm=settings.ALGORITHM
    )
    return encoded_jwt
