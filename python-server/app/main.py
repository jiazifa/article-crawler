# -*- coding: utf-8 -*-

import os

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

__all__ = ["create_app"]


def create_app() -> FastAPI:
    app = FastAPI()
    app.add_middleware(
        CORSMiddleware,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    from .api.routes import init_app as routes_init_app

    # run migration
    migrate_result = os.system("python -m alembic upgrade head")
    if migrate_result != 0:
        print("migrate failed")

    routes_init_app(app=app)

    return app


app = create_app()
