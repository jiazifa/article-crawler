from datetime import datetime
from typing import Optional
from dataclasses import dataclass
from libs.schema import PageRequest


@dataclass
class Author:
    name: str
    email: Optional[str] = None
    uri: Optional[str] = None


@dataclass
class RssEntry:
    title: str
    link: str
    #  'summary_detail': {'type': 'text/html', 'language': None, 'base': '', 'value': '千呼万唤始出来的图标变色功能，怎么就翻车了？<a href="https://sspai.com/prime/story/ios-18-tinted-icon-issues" target="_blank">查看全文</a><p>本文为会员文章，出自<a href="https://sspai.com/prime/precog/single" target="_blank">《单篇文章》</a>，订阅后可阅读全文。</p>'}
    summary: str
    summary_pure_text: str
    authors: list[Author]
    published_at: datetime


@dataclass
class RssRoot:
    title: str
    link: str
    authors: list[Author]
    entries: list[RssEntry]
    published: datetime
    # 副标题
    subtitle: Optional[str] = None
    # 图标
    icon: Optional[str] = None
    # 服务站点链接
    site_href: Optional[str] = None
    # 语言 zh-CN
    language: Optional[str] = None
    # rss20
    version: Optional[str] = None


@dataclass
class QuerySubscriptionOption:
    # page
    page: PageRequest
    # id
    ids: Optional[list[int]] = None
    # 标题
    title: Optional[str] = None
    # language
    languages: Optional[list[str]] = None


@dataclass
class QueryRssEntitykOption:
    # page
    page: PageRequest
    # id
    ids: Optional[list[int]] = None
    # 标题
    title: Optional[str] = None
