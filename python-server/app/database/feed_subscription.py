from datetime import datetime
from typing import TYPE_CHECKING, Optional

from sqlalchemy import (
    DateTime,
    Integer,
    Sequence,
    Text,
    Float,
    String,
)
from sqlalchemy.orm import Mapped, mapped_column, relationship


from ._base_model import DBBaseModel

if TYPE_CHECKING:
    from .feed_link import FeedLinkInDB
    from .feed_subscription_build_record import FeedSubscriptionBuildRecordInDB


class FeedSubscriptionInDB(DBBaseModel):
    __tablename__ = "feed_subscription"

    subscription_id: Mapped[int] = mapped_column(
        Integer,
        Sequence(start=1, increment=1, name="db_feed_subscription_id_seq"),
        primary_key=True,
        comment="subscription id",
    )

    title: Mapped[str] = mapped_column(
        String(255), nullable=False, comment="subscription title"
    )

    description: Mapped[str] = mapped_column(
        Text, nullable=True, comment="subscription description"
    )

    link: Mapped[str] = mapped_column(Text, nullable=False, comment="信息源的链接")

    site_link: Mapped[str] = mapped_column(
        Text, nullable=True, comment="源的站点链接，如果没有则为空"
    )

    logo: Mapped[str] = mapped_column(Text, nullable=True, comment="源的logo")

    language: Mapped[str] = mapped_column(String(64), nullable=True, comment="源的语言")

    rating: Mapped[str] = mapped_column(
        Float, nullable=True, comment="源的评分, 百分制"
    )

    visual_img_url: Mapped[str] = mapped_column(
        Text, nullable=True, comment="源的视觉链接，装饰性的图片链接"
    )

    sort_order: Mapped[int] = mapped_column(
        Integer, nullable=False, comment="subscription sort order"
    )

    last_build_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=True, comment="last build time"
    )

    published_at: Mapped[datetime] = mapped_column(
        DateTime, default=datetime.now, comment="create time"
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

    links: Mapped[list["FeedLinkInDB"]] = relationship(
        "FeedLinkInDB", secondary="feed_subscription_link", lazy="joined", uselist=True
    )

    build_records: Mapped[list["FeedSubscriptionBuildRecordInDB"]] = relationship(
        "FeedSubscriptionBuildRecordInDB", back_populates="subscription"
    )

    def __init__(
        self,
        title: str,
        link: str,
        description: Optional[str] = None,
        site_link: Optional[str] = None,
        logo: Optional[str] = None,
        language: Optional[str] = None,
        rating: Optional[float] = None,
        visual_img_url: Optional[str] = None,
        sort_order: Optional[int] = None,
        published_at: Optional[datetime] = None,
        last_build_at: Optional[datetime] = None,
    ):
        self.title = title
        self.description = description
        self.link = link
        self.site_link = site_link
        self.logo = logo
        self.language = language
        self.rating = rating
        self.visual_img_url = visual_img_url
        self.sort_order = sort_order
        self.published_at = published_at if published_at is not None else datetime.now()
        self.last_build_at = last_build_at
