from datetime import datetime
from enum import IntEnum

from extensions.ext_database import db
from sqlalchemy import SMALLINT, DateTime, Integer, Sequence, ForeignKey
from sqlalchemy.orm import Mapped, mapped_column


class UpdateResult(IntEnum):
    """Update result"""

    SUCCESS = 1
    FAILED = 0


# 订阅源的更新记录
class FeedSubscriptionUpdateRecordInDB(db.Model):
    __tablename__ = "feed_subs_update_record"

    # 以订阅源id为主键
    record_id: Mapped[int] = mapped_column(
        Integer,
        Sequence("feed_subs_update_record_id_seq"),
        nullable=False,
        primary_key=True,
        comment="Update record ID",
    )

    subscription_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("feed_subscription.subscription_id"),
        nullable=False,
        comment="Subscription ID",
    )
    # 更新结果
    update_result: Mapped[UpdateResult] = mapped_column(
        SMALLINT, nullable=False, comment="Update result"
    )
    # 更新时间
    updated_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, comment="Update time"
    )
