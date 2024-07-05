from datetime import datetime
from enum import IntEnum
from typing import Optional

from sqlalchemy import (
    DateTime,
    ForeignKey,
    Integer,
    Sequence,
    String,
)
from sqlalchemy.orm import Mapped, mapped_column


from ._base_model import DBBaseModel


class FeedSubscriptionBuildRecordResult(IntEnum):
    # 订阅构建结果
    UNKNOWN = 0
    # 1. 失败
    FAILED = 1
    # 2. 无更新
    NO_UPDATE = 2
    # 3. 部分更新
    SOME_UPDATE = 3
    # 4. 成功
    SUCCESS = 4


class FeedSubscriptionBuildRecordInDB(DBBaseModel):
    __table_name__ = "feed_subscription_build_record"

    record_id: Mapped[int] = mapped_column(
        Integer,
        Sequence(start=1, increment=1, name="db_feed_subscription_build_record_id_seq"),
        primary_key=True,
        comment="record id",
    )

    subscription_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("feed_subscription.subscription_id", ondelete="CASCADE"),
        comment="subscription id",
    )

    result: Mapped[FeedSubscriptionBuildRecordResult] = mapped_column(
        Integer,
        nullable=False,
        default=FeedSubscriptionBuildRecordResult.UNKNOWN,
        comment="构建结果",
    )

    remark: Mapped[str] = mapped_column(String(255), nullable=True, comment="构建备注")

    build_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=datetime.now, comment="构建时间"
    )

    def __init__(
        self,
        subscription_id: int,
        result: FeedSubscriptionBuildRecordResult = FeedSubscriptionBuildRecordResult.UNKNOWN,
        remark: Optional[str] = None,
        build_at: datetime = datetime.now(),
    ):
        self.subscription_id = subscription_id
        self.result = result
        self.remark = remark
        self.build_at = build_at
