from fastapi import APIRouter, FastAPI


def init_app(app: FastAPI) -> None:
    from . import health, user

    name_prefix = "/api"
    root_ = APIRouter(prefix=name_prefix)

    root_.include_router(health.router)
    root_.include_router(user.router)

    app.include_router(root_)
