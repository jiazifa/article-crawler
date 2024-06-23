from typing import Generator

import pytest
from flask import Flask
from flask.testing import FlaskClient, FlaskCliRunner
from app import create_app
from config import Config


class TestConfig(Config):
    def __init__(self) -> None:
        super().__init__()
        self.SQLALCHEMY_DATABASE_URI = "sqlite:///:memory:"


@pytest.fixture()
def app() -> Generator[Flask, None, None]:
    app = create_app(test_config=TestConfig())
    app.config.update(
        {
            "TESTING": True,
        }
    )

    # other setup can go here
    with app.app_context():
        from extensions.ext_database import db

        db.create_all()

    yield app

    # clean up / reset resources here


@pytest.fixture()
def client(app: Flask) -> FlaskClient:
    return app.test_client()


@pytest.fixture()
def runner(app: Flask) -> FlaskCliRunner:
    return app.test_cli_runner()


@pytest.fixture()
def db_session(app: Flask) -> Generator:
    with app.app_context():
        from extensions.ext_database import db

        session = db.session
        yield session
        session.rollback()
        session.close()


@pytest.fixture()
def admin_account() -> dict:
    return {
        "username": "admin",
        "password": "admin",
    }
