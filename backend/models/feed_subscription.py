from datetime import datetime
from enum import IntEnum
from typing import Optional, TYPE_CHECKING

from extensions.ext_database import db
from sqlalchemy import (
    SMALLINT,
    DateTime,
    ForeignKey,
    Integer,
    Sequence,
    String,
    Text,
    Column,
)
from sqlalchemy.orm import Mapped, mapped_column, relationship

if TYPE_CHECKING:
    from .feed_subs_meta import FeedSubscriptionMetaInDB
    from .feed_link import FeedLinkInDB
    from .feed_subs_update_record import FeedSubscriptionUpdateRecordInDB


class SubscriptionStatus(IntEnum):
    """Subscription status"""

    ACTIVE = 1
    INACTIVE = 0


class FeedSubscriptionInDB(db.Model):
    __tablename__ = "feed_subscription"

    subscription_id: Mapped[int] = mapped_column(
        Integer,
        Sequence("feed_subscription_id_seq"),
        ForeignKey("feed_subscription.subscription_id"),
        nullable=False,
        primary_key=True,
        comment="Subscription ID",
    )

    # 标题
    title: Mapped[str] = mapped_column(
        String(255), nullable=False, comment="Subscription title"
    )
    # 描述
    subtitle: Mapped[str] = mapped_column(
        String(255), nullable=True, comment="Subscription description"
    )
    # feed link
    link: Mapped[str] = mapped_column(Text, nullable=False, comment="Subscription link")
    # site link
    site_link: Mapped[str] = mapped_column(
        String(255), nullable=False, comment="Subscription site link"
    )
    # 图标
    icon: Mapped[str] = mapped_column(Text, nullable=True, comment="Subscription icon")
    # 语言
    language_code: Mapped[str] = mapped_column(
        String(10), nullable=True, comment="Subscription language code"
    )
    # 评分
    rating: Mapped[int] = mapped_column(
        SMALLINT, nullable=True, comment="Subscription rating, 1000 rate"
    )
    # 发布时间
    published_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, comment="Subscription published at"
    )
    # 收录时间
    created_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=datetime.now, comment="Subscription create at"
    )
    # 更新时间
    updated_at: Mapped[datetime] = mapped_column(
        DateTime,
        nullable=False,
        default=datetime.now,
        onupdate=datetime.now,
        comment="Subscription update at",
    )

    # 订阅下的链接列表
    links: Mapped[list["FeedLinkInDB"]] = relationship(
        "FeedLinkInDB",
        secondary="feed_link_subscription",
        back_populates="subscription",
    )

    # 元信息
    meta: Mapped["FeedSubscriptionMetaInDB"] = relationship(
        "FeedSubscriptionMetaInDB",
    )

    # 更新记录
    update_record: Mapped[list["FeedSubscriptionUpdateRecordInDB"]] = relationship(
        "FeedSubscriptionUpdateRecordInDB", lazy="dynamic"
    )

    def __init__(
        self,
        /,
        title: str,
        link: str,
        subtitle: Optional[str] = None,
        icon: Optional[str] = None,
        site_link: Optional[str] = None,
        published_at: Optional[datetime] = None,
    ) -> None:
        self.title = title
        self.link = link
        self.site_link = site_link
        self.published_at = published_at
        self.subtitle = subtitle
        self.icon = icon


link_subscription = db.Table(
    "feed_link_subscription",
    Column("link_id", Integer, ForeignKey("feed_link.link_id")),
    Column("subscription_id", Integer, ForeignKey("feed_subscription.subscription_id")),
)
