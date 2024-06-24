from datetime import datetime
from typing import Optional

from extensions.ext_database import db
from sqlalchemy import DateTime, ForeignKey, Integer, Boolean
from sqlalchemy.orm import Mapped, mapped_column


class FeedSubscriptionMetaInDB(db.Model):
    __tablename__ = "feed_subs_meta"

    # 以订阅源id为主键
    subscription_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("feed_subscription.subscription_id"),
        nullable=False,
        primary_key=True,
        comment="Subscription ID",
    )

    # 更新频率, 以分钟为单位，最小值为 10
    update_frequency: Mapped[int] = mapped_column(
        Integer, nullable=True, comment="Update frequency"
    )
    # 是否是动态的频率
    is_dynamic_freq: Mapped[bool] = mapped_column(
        Boolean, nullable=True, default=True, comment="Is dynamic frequency"
    )

    # 最后更新时间
    last_update_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=True, comment="Last update time"
    )

    def __init__(
        self,
        subscription_id: int,
        update_frequency: Optional[int] = None,
        last_update_at: Optional[datetime] = None,
    ) -> None:
        self.subscription_id = subscription_id
        self.update_frequency = update_frequency
        self.last_update_at = last_update_at
