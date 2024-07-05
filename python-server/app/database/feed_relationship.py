from sqlalchemy import (
    ForeignKey,
    Integer,
    Sequence,
)
from sqlalchemy.orm import Mapped, mapped_column


from ._base_model import DBBaseModel


class FeedCategorySubscriptionInDB(DBBaseModel):
    __tablename__ = "feed_category_subscription"

    id: Mapped[int] = mapped_column(
        Integer,
        Sequence(start=1, increment=1, name="db_feed_category_subscription_id_seq"),
        primary_key=True,
        comment="subscription id",
    )

    subscription_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("feed_subscription.subscription_id", ondelete="CASCADE"),
        comment="subscription id",
    )

    category_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("feed_category.category_id", ondelete="CASCADE"),
        comment="category id",
    )


class FeedSubscriptionLinkInDB(DBBaseModel):
    __tablename__ = "feed_subscription_link"

    id: Mapped[int] = mapped_column(
        Integer,
        Sequence(start=1, increment=1, name="db_feed_subscription_link_id_seq"),
        primary_key=True,
        comment="subscription link id",
    )

    subscription_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("feed_subscription.subscription_id", ondelete="CASCADE"),
        comment="subscription id",
    )

    link_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("feed_link.link_id", ondelete="CASCADE"),
        comment="link id",
    )
