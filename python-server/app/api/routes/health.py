from typing import Dict

from fastapi import APIRouter

from app.api.response import Response

router = APIRouter(prefix="/health", tags=["health"])


@router.get("/check")
async def health_check_get() -> Response[Dict[str, str]]:
    """健康检查"""
    return Response[dict](data={"status": "ok"})
