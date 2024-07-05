import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession


async def setup_data(session: AsyncSession) -> None:
    from app.database import UserInDB

    user1 = UserInDB(username="user1", phone="12345678901")
    user2 = UserInDB(username="user2", phone="12345678902")
    user3 = UserInDB(username="user3", phone="12345678903")
    session.add_all([user1, user2, user3])

    await session.flush()

    await session.commit()


@pytest.mark.anyio
async def test_register_user(ac: AsyncClient, session: AsyncSession) -> None:
    from app.api.response import Response

    await setup_data(session)

    data = {"username": "test", "phone": "12345678904"}
    response = await ac.post(
        "/auth/register",
        json=data,
    )
    assert 200 == response.status_code
    assert Response[int].model_validate(response.json()).data is not None
