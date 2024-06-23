import json
import logging
import os
from typing import Optional, TYPE_CHECKING

from flask import Flask, Response, Request, Blueprint
from flask_cors import CORS

from config import Config
from extensions import ext_database, ext_migrate, ext_rest
from extensions.ext_database import db


def init_extensions(app: Flask) -> None:
    ext_database.init_app(app)
    ext_migrate.init(app, db)
    ext_rest.init_app(app)


def register_blueprints(app: Flask) -> None:
    from views import init_app as init_views

    init_views(app)
    return None


def register_commands(app) -> None: ...


def create_app(test_config=None) -> Flask:
    app = Flask(__name__)

    if test_config:
        app.config.from_object(test_config)
    else:
        app.config.from_object(Config())

    app.secret_key = app.config["SECRET_KEY"]

    logging.basicConfig(level=app.config.get("LOG_LEVEL", "INFO"))

    init_extensions(app)
    register_blueprints(app)
    register_commands(app)

    return app


app = create_app()
# celery = app.extensions["celery"]

if app.config["TESTING"]:
    print("App is running in TESTING mode")


@app.after_request
def after_request(response: Response) -> Response:
    """Add Version headers to the response."""
    response.set_cookie("remember_token", "", expires=0)
    response.headers.add("X-Version", app.config["CURRENT_VERSION"])
    response.headers.add("X-Env", app.config["DEPLOY_ENV"])
    return response


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5001)
