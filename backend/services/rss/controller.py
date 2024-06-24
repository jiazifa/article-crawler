from typing import Optional
from datetime import datetime
import time
from logging import getLogger
import requests
from sqlalchemy import select
from sqlalchemy.orm import scoped_session
from feedparser import FeedParserDict, parse as feed_parser_func
from dateparser import parse as date_parser_func
from libs.helper import contains_html_code
from models.feed_link import FeedLinkInDB
from models.feed_subscription import FeedSubscriptionInDB
from models.feed_subs_meta import FeedSubscriptionMetaInDB
from .schema import (
    RssRoot,
    RssEntry,
    Author,
    QuerySubscriptionOption,
    QueryRssEntitykOption,
)
from libs.schema import PageResponse

logger = getLogger(__name__)


class _RssController:
    @staticmethod
    def _parser_authors_from_entity(entity: dict[str, any]) -> list[Author]:
        authors: list[Author] = []
        if "author_detail" in entity:
            author_detail = entity["author_detail"]
            authors.append(
                Author(name=author_detail["name"], email=author_detail.get("email"))
            )

        authors = [author for author in authors if author.name]
        return authors

    @staticmethod
    def _parser_summary_from_entity(entity: dict[str, any]) -> str:
        summary: str = ""
        if "summary" in entity and not contains_html_code(entity["summary"]):
            summary = entity["summary"]
        elif "summary_detail" in entity:
            summary_detail: dict[str, str] = entity["summary_detail"]
            summary_ = summary_detail["value"]
            summary = summary_
        return summary


class RssController:
    def __init__(self, session: scoped_session) -> None:
        self.session = session

    @staticmethod
    def parser_feed_from_url(url: str) -> Optional[FeedParserDict]:
        resp = requests.get(url)
        if not resp.ok:
            return None
        try:
            rss = feed_parser_func(resp.text)
        except Exception:
            logger.error(f"parser feed from url error: {url}")
            return None
        return rss

    def insert_subscription_from_url(self, url: str) -> Optional[FeedSubscriptionInDB]:
        session = self.session
        # query by url
        exsits_stmt = select(FeedSubscriptionInDB).where(
            FeedSubscriptionInDB.link == url
        )
        exsits_model = session.execute(exsits_stmt).scalar_one_or_none()
        if exsits_model:
            return exsits_model
        rss_dict = self.parser_feed_from_url(url)
        if rss_dict is None:
            return None

        if "bozo" in rss_dict and rss_dict["bozo"]:
            return None

        rss_root = self._parser_rss_from_feed(url, rss_dict)
        if not rss_root:
            return None
        return self.insert_subscription(rss_root)

    def _parser_rss_from_feed(
        self, feed_url: str, rss: FeedParserDict
    ) -> Optional[RssRoot]:
        # {'bozo': False, 'entries': [
        # {'title': 'iOS 18 的图标自定义有什么问题？',
        # 'title_detail': {
        # 'type': 'text/plain',
        # 'language': None, 'base': '',
        # 'value': 'iOS 18 的图标自定义有什么问题？'},

        # 'links': [{'rel': 'alternate', 'type': 'text/html', 'href': 'https://sspai.com/prime/story/ios-18-tinted-icon-issues'}],
        # 'link': 'https://sspai.com/prime/story/ios-18-tinted-icon-issues',
        # 'summary': '千呼万唤始出来的图标变色功能，怎么就翻车了？',
        # 'summary_detail': {'type': 'text/html', 'language': None, 'base': '', 'value': '千呼万唤始出来的图标变色功能，怎么就翻车了？'}, 'authors': [{'name': '少数派编辑部'}],
        # 'author': '少数派编辑部', 'author_detail': {'name': '少数派编辑部'},
        # 'published': 'Fri, 21 Jun 2024 18:10:41 +0800',
        # 'published_parsed': time.struct_time(tm_year=2024, tm_mon=6, tm_mday=21, tm_hour=10, tm_min=10, tm_sec=41, tm_wday=4, tm_yday=173, tm_isdst=0)}],

        # 'feed': {'title': '少数派', 'title_detail': {'type': 'text/plain', 'language': None, 'base': '', 'value': '少数派'},
        # 'links': [{'rel': 'alternate', 'type': 'text/html', 'href': 'https://sspai.com'}, {'href': 'https://sspai.com/feed', 'type': 'application/rss+xml', 'ref': 'self', 'rel': 'alternate'},
        # {'href': 'https://sspai.com/feed', 'type': 'application/rss+xml', 'ref': 'hub', 'rel': 'alternate'}],
        # 'link': 'https://sspai.com', 'subtitle': '少数派致力于更好地运用数字产品或科学方法，帮助用户提升工作效率和生活品质',
        # 'subtitle_detail': {'type': 'text/html', 'language': None, 'base': '', 'value': '少数 派致力于更好地运用数字产品或科学方法，帮助用户提升工作效率和生活品质'},
        # 'language': 'zh-CN', 'authors': [{'name': '少数派', 'email': 'contact@sspai.com'}],
        # 'author': 'contact@sspai.com (少数派)', 'author_detail': {'name': '少数派', 'email': 'contact@sspai.com'}, 'published': 'Fri, 21 Jun 2024 18:10:41 +0800',
        # 'published_parsed': time.struct_time(tm_year=2024, tm_mon=6, tm_mday=21, tm_hour=10, tm_min=10, tm_sec=41, tm_wday=4, tm_yday=173, tm_isdst=0)},
        # 'headers': {}, 'encoding': 'utf-8', 'version': 'rss20', 'namespaces': {'': 'http://www.w3.org/2005/Atom', 'dc': 'http://purl.org/dc/elements/1.1/'}}
        item_dicts: dict[str, any] = rss["entries"]
        feed_dicts: dict[str, any] = rss["feed"]

        def _parse_feed_to_item(item: dict[str, any]) -> Optional[RssEntry]:
            title: str = item.get("title")
            link: str = item.get("link")

            if not (title and link):
                return None

            summary_ = item.get("summary", "")
            summary_text: str = _RssController._parser_summary_from_entity(item)
            authors: list[Author] = _RssController._parser_authors_from_entity(item)
            published_at: Optional[datetime] = None

            published_parsed: Optional[time.struct_time] = item.get("published_parsed")
            published: Optional[str] = item.get("published")
            if published_parsed:
                published_at = datetime.fromtimestamp(time.mktime(published_parsed))
            elif published:
                published_at = date_parser_func(published)

            return RssEntry(
                title=title,
                link=link,
                summary=summary_,
                summary_pure_text=summary_text,
                authors=authors,
                published_at=published_at,
            )

        items = [_parse_feed_to_item(item) for item in item_dicts]
        items = [item for item in items if item]

        feed_title = feed_dicts.get("title")
        site_href = feed_dicts.get("link")
        feed_link = feed_url
        feed_subtitle = feed_dicts.get("subtitle")
        feed_language = feed_dicts.get("language")

        def _parser_authors_from_feed(feed: dict[str, any]) -> list[Author]:
            authors: list[Author] = []
            author_detail: dict = feed.get("author_detail", {})
            if author_detail:
                authors.append(
                    Author(
                        name=author_detail.get("name"), email=author_detail.get("email")
                    )
                )

            # filter None if not name
            authors = [author for author in authors if author.name]
            return authors

        feed_authors = _parser_authors_from_feed(feed_dicts)
        feed_published_at: Optional[datetime] = None

        feed_published_parsed: Optional[time.struct_time] = feed_dicts.get(
            "published_parsed"
        )
        feed_published: Optional[str] = feed_dicts.get("published")
        if feed_published_parsed:
            feed_published_at = datetime.fromtimestamp(
                time.mktime(feed_published_parsed)
            )
        elif feed_published:
            feed_published_at = date_parser_func(feed_published)

        root = RssRoot(
            title=feed_title,
            subtitle=feed_subtitle,
            link=feed_link,
            icon=None,
            site_href=site_href,
            language=feed_language,
            authors=feed_authors,
            entries=items,
            published=feed_published_at,
            version=rss.get("version"),
        )
        return root

    def insert_subscription(self, root: RssRoot) -> FeedSubscriptionInDB:
        session = self.session

        # first query subscription is exist
        exsits_stmt = select(FeedSubscriptionInDB).where(
            FeedSubscriptionInDB.link == root.link
        )
        updated_model = session.execute(exsits_stmt).scalar_one_or_none()
        if not updated_model:
            new_model = FeedSubscriptionInDB(
                title=root.title,
                link=root.link,
                subtitle=root.subtitle,
                icon=None,
                site_link=root.site_href,
                published_at=root.published,
            )
            session.add(new_model)
            meta_model = FeedSubscriptionMetaInDB(
                subscription_id=new_model.subscription_id,
                update_frequency=None,
                last_update_at=datetime.now(),
            )
            new_model.meta = meta_model
            session.add(meta_model)
            updated_model = new_model
        # do update
        updated_model.title = root.title
        updated_model.subtitle = root.subtitle
        updated_model.site_link = root.site_href
        updated_model.icon = root.icon
        updated_model.language_code = root.language
        updated_model.published_at = root.published
        _ = self.insert_entities(root.entries, updated_model)
        return updated_model

    def query_subscription(
        self, option: QuerySubscriptionOption
    ) -> PageResponse[FeedSubscriptionInDB]:
        session = self.session
        page = option.page
        query = session.query(FeedSubscriptionInDB)
        if option.ids:
            query = query.filter(FeedSubscriptionInDB.subscription_id.in_(option.ids))
        if option.title:
            query = query.filter(FeedSubscriptionInDB.title.like(f"%{option.title}%"))
        if option.languages:
            query = query.filter(
                FeedSubscriptionInDB.language_code.in_(option.languages)
            )

        query = query.order_by(FeedSubscriptionInDB.published_at.desc())
        total = query.count()
        query = query.offset(page.get_offset()).limit(page.get_limit())

        items = query.all()
        return PageResponse.by_items(data=items, total_count=total, page=page)

    def update_subscription_meta(
        self,
        subscription: FeedSubscriptionInDB,
        update_frequency: Optional[int] = None,
        last_update_at: Optional[datetime] = None,
    ) -> None:
        meta = subscription.meta
        if update_frequency is not None:
            meta.update_frequency = update_frequency
        if last_update_at is not None:
            meta.last_update_at = last_update_at
        self.session.commit()

    def is_subscription_exist(self, link: str) -> bool:
        session = self.session
        exsits_stmt = select(FeedSubscriptionInDB).where(
            FeedSubscriptionInDB.link == link
        )
        return session.execute(exsits_stmt).scalar_one_or_none() is not None

    def insert_entities(
        self, entities: list[RssEntry], subscription: FeedSubscriptionInDB
    ) -> list[FeedLinkInDB]:
        """
        将给定的 RssEntry 实体列表插入到数据库中，并与指定的订阅ID关联。

        Args:
            entities (list[RssEntry]): 包含要插入的 RssEntry 实体的列表。
            subscription (FeedSubscriptionInDB): 要关联的订阅实体。

        Returns:
            list[FeedLinkInDB]: 插入到数据库中的 FeedLinkInDB 实体列表。
        """

        session = self.session

        def _insert_link(entity: RssEntry) -> FeedLinkInDB:
            link = entity.link
            exists_stmt = select(FeedLinkInDB).where(FeedLinkInDB.link == link)
            updated_model = session.execute(exists_stmt).scalar_one_or_none()
            if not updated_model:
                new_model = FeedLinkInDB(
                    title=entity.title,
                    link=entity.link,
                    published_at=entity.published_at,
                )
                updated_model = new_model
            updated_model.title = entity.title
            updated_model.published_at = entity.published_at
            updated_model.description = entity.summary
            updated_model.desc_pure_text = entity.summary_pure_text
            return updated_model

        items = [_insert_link(entity) for entity in entities]
        for item in items:
            subscription.links.append(item)

        session.flush(items)
        session.commit()

        return items

    def query_entities(
        self, option: QueryRssEntitykOption
    ) -> PageResponse[FeedLinkInDB]:
        session = self.session
        page = option.page
        query = session.query(FeedLinkInDB)
        if option.ids:
            query = query.filter(FeedLinkInDB.link_id.in_(option.ids))
        if option.title:
            query = query.filter(FeedLinkInDB.title.like(f"%{option.title}%"))

        query = query.order_by(FeedLinkInDB.published_at.desc())
        total = query.count()
        query = query.offset(page.get_offset()).limit(page.get_limit())

        items = query.all()
        return PageResponse.by_items(data=items, total_count=total, page=page)
