from typing import List, Literal, Optional

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.database import PermissionInDB, PermissionRoleInDB, RoleInDB, UserRoleInDB


# 创建权限
async def create_permission(
    session: AsyncSession, identifier: str, name: str, remark: Optional[str] = None
) -> PermissionInDB:
    # 查询权限是否存在
    exsits_stmt = select(PermissionInDB).where(PermissionInDB.identifier == identifier)
    is_exist = await session.scalar(exsits_stmt)
    if is_exist:
        return is_exist
    permission = PermissionInDB(
        identifier=identifier, permission_name=name, remark=remark
    )
    session.add(permission)
    await session.flush()
    return permission


# 创建一个角色
async def create_role(session: AsyncSession, identifier: str, name: str) -> RoleInDB:
    # 查询角色是否存在
    exsits_stmt = select(RoleInDB).where(RoleInDB.identifier == identifier)
    is_exist = await session.scalar(exsits_stmt)
    if is_exist:
        return is_exist
    role = RoleInDB(identifier=identifier, role_name=name)
    session.add(role)
    await session.flush()
    return role


# 获取用户角色
async def get_user_role(session: AsyncSession, user_id: int) -> List[RoleInDB]:
    # RoleInDB join UserRoleInDB
    role_stmt = (
        select(RoleInDB)
        .join(UserRoleInDB, UserRoleInDB.role_id == RoleInDB.id)
        .where(UserRoleInDB.user_id == user_id)
    )
    result = await session.execute(role_stmt)
    roles = result.scalars().all()
    return roles


async def grant_user_role(
    session: AsyncSession, admin_id: int, role_id: int
) -> UserRoleInDB:
    # 赋予某一个用户的角色
    exsits_stmt = select(UserRoleInDB).where(
        UserRoleInDB.user_id == admin_id, UserRoleInDB.role_id == role_id
    )
    is_exist = await session.scalar(exsits_stmt)
    if is_exist:
        return is_exist
    user_role = UserRoleInDB(user_id=admin_id, role_id=role_id)
    session.add(user_role)
    await session.flush()
    return user_role


async def revoke_user_role(
    session: AsyncSession, admin_id: int, role_id: int
) -> UserRoleInDB:
    # 移除某一个用户的角色
    user_role_stmt = select(UserRoleInDB).where(
        UserRoleInDB.user_id == admin_id, UserRoleInDB.role_id == role_id
    )
    user_role = await session.scalar(user_role_stmt)
    if not user_role:
        raise ValueError("user role not exists")
    session.delete(user_role)
    await session.flush()
    return user_role


async def update_role_permissions(
    session: AsyncSession,
    role_id: int,
    pendding_permission_ids: List[int],
    action: Literal["add", "remove"] = "add",
) -> List[PermissionRoleInDB]:
    # pendding_permission_ids 不能为空，不能包含空
    if not pendding_permission_ids:
        raise ValueError("permission ids can not be empty")
    pendding_permission_ids = list(set(pendding_permission_ids))
    pendding_permission_ids = [p for p in pendding_permission_ids if p]
    if not pendding_permission_ids:
        raise ValueError("permission ids can not be empty")
    # 更新角色的权限 / 新增 / 删除
    permissions = []
    for permission_id in pendding_permission_ids:
        if action == "add":
            exsits_stmt = select(PermissionRoleInDB).where(
                PermissionRoleInDB.role_id == role_id,
                PermissionRoleInDB.permission_id == permission_id,
            )
            is_exist = await session.scalar(exsits_stmt)
            if is_exist:
                continue

            permission = PermissionRoleInDB(
                role_id=role_id, permission_id=permission_id
            )
            session.add(permission)
            await session.flush()
            permissions.append(permission)
        elif action == "remove":
            permission_stmt = (
                session.query(PermissionRoleInDB)
                .filter(
                    PermissionRoleInDB.role_id == role_id,
                    PermissionRoleInDB.permission_id == permission_id,
                )
                .first()
            )
            result = await session.execute(permission_stmt)

            permission = await result.scalars().first()
            if permission:
                await session.delete(permission)
                permissions.append(permission)
    await session.commit()
    return permissions


# 获取用户权限
async def get_user_permissions(
    session: AsyncSession, user_id: int
) -> List[PermissionInDB]:
    # RoleInDB join UserRoleInDB join PermissionRoleInDB
    stmt = (
        select(PermissionInDB)
        .join(PermissionRoleInDB, PermissionRoleInDB.role_id == RoleInDB.id)
        .join(UserRoleInDB, UserRoleInDB.role_id == RoleInDB.id)
        .where(UserRoleInDB.user_id == user_id)
    )
    result = await session.execute(stmt)
    permissions = result.scalars().all()
    return permissions


# 判断用户是否有权限
async def user_has_permission(
    session: AsyncSession, user_id: int, permission: str
) -> bool:
    # RoleInDB join UserRoleInDB join PermissionRoleInDB
    permissions = get_user_permissions(session=session, user_id=user_id)
    for p in permissions:
        if p.permission_name == permission:
            return True
    return False
