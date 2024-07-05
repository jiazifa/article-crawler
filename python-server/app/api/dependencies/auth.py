from typing import Annotated, Optional

from fastapi import Depends, Request

from app.api.error import AuthenticationException
from app.database import UserInDB
from app.utils import get_logger

from .session import AsyncSession

logger = get_logger(__name__)


def _get_token(request: Request) -> Optional[str]:
    auth_content = request.headers.get("Authorization")
    # tirm `Bearer `
    content = auth_content.removeprefix("Bearer ") if auth_content else None
    return content


async def get_avaliable_user_if_could(
    session: AsyncSession, token: Optional[str] = Depends(_get_token)
) -> Optional[UserInDB]:
    if not token:
        return None

    user = await UserInDB.find_by_token(session=session, token=token)
    return user


async def get_avaliable_user_with_raise(
    session: AsyncSession,
    token: Optional[str] = Depends(_get_token),
) -> UserInDB:
    user = await get_avaliable_user_if_could(session=session, token=token)
    if not user:
        raise AuthenticationException("user not found")
    return user


UserOption = Annotated[Optional[UserInDB], Depends(get_avaliable_user_if_could)]

UserRequire = Annotated[UserInDB, Depends(get_avaliable_user_with_raise)]
