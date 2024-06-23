from dataclasses import dataclass
from typing import Generic, TypeVar


@dataclass
class PageRequest:
    page: int
    page_size: int

    @staticmethod
    def default() -> "PageRequest":
        return PageRequest(page=1, page_size=10)

    def get_offset(self) -> int:
        return (self.page - 1) * self.page_size

    def get_limit(self) -> int:
        return self.page_size


PageRespItem = TypeVar("PageRespItem")


@dataclass
class PageResponse(Generic[PageRespItem]):
    total_page: int
    current_page: int
    page_size: int
    data: list[PageRespItem]

    @staticmethod
    def by_items(
        data: list[PageRespItem], total_count: int, page: PageRequest
    ) -> "PageResponse":
        total_page = total_count // page.page_size
        if total_count % page.page_size:
            total_page += 1
        return PageResponse(
            total_page=total_page,
            current_page=page.page,
            page_size=page.page_size,
            data=data,
        )
