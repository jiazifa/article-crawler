from datetime import datetime
from typing import TYPE_CHECKING, Optional

from sqlalchemy import (
    DateTime,
    JSON,
    Integer,
    Sequence,
    Text,
    String,
)
from sqlalchemy.orm import Mapped, mapped_column, relationship


from ._base_model import DBBaseModel

if TYPE_CHECKING:
    from .feed_subscription import FeedSubscriptionInDB


class FeedLinkInDB(DBBaseModel):
    __table_name__ = "feed_link"

    link_id: Mapped[int] = mapped_column(
        Integer,
        Sequence(start=1, increment=1, name="db_feed_link_id_seq"),
        primary_key=True,
        comment="link id",
    )

    title: Mapped[str] = mapped_column(
        String(255), nullable=False, comment="link title"
    )

    link: Mapped[str] = mapped_column(Text, nullable=False, comment="链接的地址")

    description: Mapped[str] = mapped_column(
        Text, nullable=True, comment="链接的描述, 可能包含html"
    )

    desc_pure_text: Mapped[str] = mapped_column(
        Text, nullable=True, comment="链接的描述纯文本"
    )

    images: Mapped[list[dict]] = mapped_column(
        JSON, nullable=True, comment="链接的图片, 通常是一个 list[dict]"
    )
    authors: Mapped[list[dict]] = mapped_column(
        JSON, nullable=True, comment="链接的作者, 通常是一个 list[dict]"
    )

    tags: Mapped[list[str]] = mapped_column(
        JSON, nullable=True, comment="链接的标签, 通常是一个 list[str]"
    )

    published_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=True, default=datetime.now, comment="发布时间"
    )

    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=datetime.now, comment="create time"
    )

    updated_at: Mapped[datetime] = mapped_column(
        DateTime,
        nullable=True,
        default=datetime.now,
        onupdate=datetime.now,
        comment="update time",
    )

    subscriptions: Mapped[list["FeedSubscriptionInDB"]] = relationship(
        "FeedSubscriptionInDB",
        secondary="feed_subscription_link",
        back_populates="links",
        uselist=True,
    )

    def __init__(
        self,
        title: str,
        link: str,
        description: Optional[str] = None,
        desc_pure_text: Optional[str] = None,
        images: Optional[list[dict]] = None,
        authors: Optional[list[dict]] = None,
        tags: Optional[list[str]] = None,
        published_at: Optional[datetime] = None,
    ):
        self.title = title
        self.link = link
        self.description = description
        self.desc_pure_text = desc_pure_text
        self.images = images
        self.authors = authors
        self.tags = tags
        self.published_at = published_at
