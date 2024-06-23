from datetime import datetime
from typing import Optional, TYPE_CHECKING

from extensions.ext_database import db
from sqlalchemy import (
    DateTime,
    ForeignKey,
    Integer,
    Sequence,
    String,
    Text,
    JSON,
)
from sqlalchemy.orm import Mapped, mapped_column, relationship

if TYPE_CHECKING:
    from .feed_subscription import FeedSubscriptionInDB


class FeedLinkInDB(db.Model):
    __tablename__ = "feed_link"

    link_id: Mapped[int] = mapped_column(
        Integer,
        Sequence("feed_link_id_seq"),
        ForeignKey("feed_link.link_id"),
        nullable=False,
        primary_key=True,
        comment="Link ID",
    )
    title: Mapped[str] = mapped_column(
        String(255), nullable=False, comment="Feed title"
    )
    link: Mapped[str] = mapped_column(Text, nullable=False, comment="Feed link")
    # 描述 / 可能包含 html 标签
    description: Mapped[str] = mapped_column(
        Text, nullable=False, comment="Feed description"
    )
    # 纯文本描述
    desc_pure_text: Mapped[str] = mapped_column(
        Text, nullable=True, comment="Feed description pure text"
    )
    # 标签
    tags: Mapped[list[str]] = mapped_column(
        JSON, nullable=True, comment="Feed tags", index=True
    )
    # 图片集合，可能为空 形式是 json 的 集合
    images: Mapped[dict[str, any]] = mapped_column(
        JSON, nullable=True, comment="Feed images"
    )
    # 作者集合，可能为空，形式是 json 的 集合
    authors: Mapped[dict[str, any]] = mapped_column(
        JSON, nullable=True, comment="Feed authors"
    )
    # 发布时间，单位是秒
    published_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, comment="Feed published at"
    )
    # 收录时间
    created_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=datetime.now, comment="Feed create at"
    )
    # 更新时间
    updated_at: Mapped[datetime] = mapped_column(
        DateTime,
        nullable=False,
        default=datetime.now,
        onupdate=datetime.now,
        comment="Feed updated at",
    )

    subscription: Mapped[Optional["FeedSubscriptionInDB"]] = relationship(
        "FeedSubscriptionInDB",
        secondary="feed_link_subscription",
        back_populates="links",
    )

    def __init__(
        self,
        title: str,
        link: str,
        published_at: datetime,
        description: Optional[str] = None,
        pure_text: Optional[str] = None,
        images: Optional[dict[str, any]] = None,
        authors: Optional[dict[str, any]] = None,
    ):
        self.title = title
        self.link = link
        self.description = description
        self.desc_pure_text = pure_text
        self.images = images
        self.authors = authors
        self.published_at = published_at
