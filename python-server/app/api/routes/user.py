from fastapi import APIRouter

from app.api.dependencies.auth import UserRequire
from app.api.dependencies.session import AsyncSession
from app.api.response import Response
from app.controllers.user import (LoginResp, LoginUser, RegisterUser, UserSchema,
                                  login_user, register_user, user_count)

router = APIRouter(prefix="/user", tags=["个人信息接口"])


@router.post(
    "/register",
    summary="注册",
    description="注册用户",
    response_model=Response[int],
)
async def _register_user(req: RegisterUser, session: AsyncSession) -> Response[int]:
    print(f"session: {session}")
    try:
        user = await register_user(create_user=req, session=session)
        return Response(data=user.id)
    except Exception as e:
        print(f"register failed: {e}")
        return Response.from_error("register failed")


@router.post(
    "/login",
    summary="登录",
    description="登录",
    response_model=Response[LoginResp],
)
async def _user_login(req: LoginUser, session: AsyncSession) -> Response[LoginResp]:
    user = await login_user(login_user=req, session=session)
    if not user:
        return Response.from_error(message="login failed")
    resp = LoginResp(token=user.token.token, user=UserSchema.model_validate(user))
    return Response(data=resp)


@router.post(
    "/info",
    summary="用户信息",
    description="获得用户信息",
    response_model=Response[UserSchema],
)
async def customer_info(
    user: UserRequire,
) -> Response[UserSchema]:
    u = UserSchema.model_validate(user)
    return Response(data=u)


@router.post(
    "/logout",
    summary="登出",
    description="登出",
    response_model=Response[bool],
)
async def user_logout() -> Response[bool]:
    return Response(data=True)


@router.get(
    "/count",
    summary="用户数量",
    description="用户数量",
)
async def _user_count(session: AsyncSession) -> Response[int]:
    count = await user_count(session=session)
    return Response(data=count)
