from datetime import datetime
from enum import IntEnum
from typing import TYPE_CHECKING, Optional

from sqlalchemy import SMALLINT, DateTime, ForeignKey, Integer, Sequence, String
from sqlalchemy.orm import Mapped, mapped_column, relationship


from ._base_model import DBBaseModel

if TYPE_CHECKING:
    from .feed_subscription import FeedSubscriptionInDB


class FeedCategoryScope(IntEnum):
    """
    分类的作用域

    1. 官方分类
    2. 用户自定义分类

    """

    OFFICIAL = 1
    CUSTOM = 2


class FeedCategoryInDB(DBBaseModel):
    """
    订阅的分类，用于分类订阅的信息源

    默认的集合为官方的集合，用户可以自定义自己的集合

    """

    __tablename__ = "feed_category"

    category_id: Mapped[int] = mapped_column(
        Integer,
        Sequence(start=1, increment=1, name="db_feed_category_id_seq"),
        primary_key=True,
        comment="category id",
    )
    # title
    title: Mapped[str] = mapped_column(
        String(255), nullable=False, comment="category title"
    )

    # description
    description: Mapped[str] = mapped_column(
        String(255), nullable=False, comment="category description"
    )

    # parent_id
    parent_id: Mapped[Optional[int]] = mapped_column(
        Integer,
        ForeignKey("feed_category.id"),
        nullable=True,
        comment="parent category id",
    )

    # icon
    icon: Mapped[str] = mapped_column(
        String(255), nullable=True, comment="category icon"
    )
    # sort_order
    sort_order: Mapped[int] = mapped_column(
        Integer, nullable=True, comment="category sort order"
    )

    scope: Mapped[FeedCategoryScope] = mapped_column(
        SMALLINT,
        nullable=False,
        default=FeedCategoryScope.OFFICIAL,
        comment="分类的作用域 1. 官方分类 2. 用户自定义分类",
    )

    # created_at
    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=datetime.now, comment="create time"
    )
    # updated_at
    updated_at: Mapped[datetime] = mapped_column(
        DateTime,
        nullable=True,
        default=datetime.now,
        onupdate=datetime.now,
        comment="update time",
    )

    subscriptions: Mapped[list["FeedSubscriptionInDB"]] = relationship(
        "FeedSubscriptionInDB",
        secondary="feed_subscription_category",
        lazy="joined",
        uselist=True,
    )

    def __init__(
        self,
        title: str,
        description: str,
        scope: FeedCategoryScope = FeedCategoryScope.OFFICIAL,
        parent_id: Optional[int] = None,
        icon: Optional[str] = None,
        sort_order: Optional[int] = None,
    ):
        self.title = title
        self.description = description
        self.parent_id = parent_id
        self.icon = icon
        self.sort_order = sort_order
        self.scope = scope
