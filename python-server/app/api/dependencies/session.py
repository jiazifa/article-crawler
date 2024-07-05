from typing import Annotated, AsyncIterator

from fastapi import Depends
from sqlalchemy.exc import SQLAlchemyError
from sqlalchemy.ext.asyncio import async_sessionmaker

from app.database._session import AsyncSessionLocal
from app.utils import get_logger

logger = get_logger(__name__)


async def get_session() -> AsyncIterator[async_sessionmaker]:
    try:
        session = AsyncSessionLocal()
        yield session
    except SQLAlchemyError as e:
        logger.exception(e)
        raise e
    finally:
        await session.commit()
        await session.close()


AsyncSession = Annotated[async_sessionmaker, Depends(get_session)]
