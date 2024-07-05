import os
from pathlib import Path

from pydantic_settings import BaseSettings, SettingsConfigDict

env_file = Path(__file__).parent.parent.joinpath(f"{os.environ['APP_CONFIG_FILE']}.env")
content = env_file.read_text()


class Settings(BaseSettings):
    DATABASE_URL: str
    SECRET_KEY: str
    ALGORITHM: str = "HS256"
    ECHO_SQL: bool = False

    model_config = SettingsConfigDict(
        env_file=env_file,
        case_sensitive=False,
    )


settings = Settings.model_validate({})
