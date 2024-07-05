from typing import Any, Dict

from sqlalchemy.orm import declarative_base

Base = declarative_base()


class DBBaseModel(Base):
    __abstract__ = True

    def to_dict(self) -> Dict[str, Any]:
        return {c.name: getattr(self, c.name) for c in self.__table__.columns}
