import os

import dotenv

dotenv.load_dotenv()

DEFAULTS = {
    "SQLALCHEMY_DATABASE_URI": "sqlite:///db.sqlite3",
    "SQLALCHEMY_POOL_SIZE": 30,
    "SQLALCHEMY_MAX_OVERFLOW": 10,
    "SQLALCHEMY_POOL_RECYCLE": 3600,
    "SQLALCHEMY_ECHO": "False",
    "APP_WEB_URL": "http://localhost:8000",
    "SECRET_KEY": "secret-key",
    'DEPLOY_ENV': 'PRODUCTION',
}


def get_env(key):
    return os.environ.get(key, DEFAULTS.get(key))


def get_bool_env(key):
    value = get_env(key)
    return value.lower() == "true" if value is not None else False


class Config:
    def __init__(self) -> None:
        self.CURRENT_VERSION = "1.0.0"
        self.TESTING = False
        self.DEPLOY_ENV = get_env('DEPLOY_ENV')
        # log level
        self.LOG_LEVEL = get_env("LOG_LEVEL")
        # web url prefix
        self.APP_WEB_URL = get_env("APP_WEB_URL")
        # Your App secret key will be used for securely signing the session cookie
        # Make sure you are changing this key for your deployment with a strong key.
        # You can generate a strong key using `openssl rand -base64 42`.
        # Alternatively you can set it with `SECRET_KEY` environment variable.
        self.SECRET_KEY = get_env("SECRET_KEY")

        # SQLAlchemy settings
        self.SQLALCHEMY_DATABASE_URI = get_env("SQLALCHEMY_DATABASE_URI")
        
