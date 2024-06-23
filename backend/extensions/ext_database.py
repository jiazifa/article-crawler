from flask_sqlalchemy import SQLAlchemy
from sqlalchemy.orm import scoped_session

db = SQLAlchemy()

Session: scoped_session = db.session


def init_app(app):
    from models import (
        feed_link,  # noqa: F401
        feed_subs_meta,  # noqa: F401
        feed_subs_update_record,  # noqa: F401
        feed_subscription,  # noqa: F401
    )  # noqa: F401

    db.init_app(app)
