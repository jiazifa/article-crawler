import pytest
from httpx import AsyncClient


@pytest.mark.anyio
async def test_register_user(ac: AsyncClient) -> None:
    response = await ac.post(
        "/health/check",
    )
    assert 200 == response.status_code
