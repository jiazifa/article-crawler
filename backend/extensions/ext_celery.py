from flask import Flask
from celery import Celery

celery_app = Celery(__name__)


def init_app(app: Flask) -> None:
    celery_app.conf.update(app.config)
    celery_app.set_default()
    app.extensions["celery"] = celery_app
    return None
