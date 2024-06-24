"""account_tables

Revision ID: 9d59739f8edc
Revises: 
Create Date: 2024-04-19 23:52:01.944467

"""
from alembic import op
import sqlalchemy as sa


# revision identifiers, used by Alembic.
revision = '9d59739f8edc'
down_revision = None
branch_labels = None
depends_on = None


def upgrade():
    # ### commands auto generated by Alembic - please adjust! ###
    op.create_table('db_user_role',
    sa.Column('id', sa.Integer(), nullable=False, comment='user_role id'),
    sa.Column('user_id', sa.Integer(), nullable=False, comment='user id'),
    sa.Column('role_id', sa.Integer(), nullable=False, comment='admin_role id'),
    sa.Column('created_at', sa.DateTime(), nullable=False, comment='创建时间'),
    sa.Column('updated_at', sa.DateTime(), nullable=True, comment='更新时间'),
    sa.PrimaryKeyConstraint('id')
    )
    op.create_table('permission',
    sa.Column('id', sa.Integer(), nullable=False, comment='permission id'),
    sa.Column('identifier', sa.String(length=64), nullable=False, comment='权限标识'),
    sa.Column('permission_name', sa.String(length=32), nullable=False, comment='权限名称'),
    sa.Column('remark', sa.String(length=100), nullable=True, comment='备注'),
    sa.Column('enabled', sa.SMALLINT(), nullable=True, comment='是否启用, 0 禁用 1 启用 2 废弃'),
    sa.Column('created_at', sa.DateTime(), nullable=False, comment='创建时间'),
    sa.Column('updated_at', sa.DateTime(), nullable=False, comment='更新时间'),
    sa.PrimaryKeyConstraint('id')
    )
    with op.batch_alter_table('permission', schema=None) as batch_op:
        batch_op.create_index(batch_op.f('ix_permission_identifier'), ['identifier'], unique=False)

    op.create_table('permission_role',
    sa.Column('id', sa.Integer(), nullable=False, comment='permission_role id'),
    sa.Column('role_id', sa.Integer(), nullable=False, comment='角色标识'),
    sa.Column('permission_id', sa.Integer(), nullable=False, comment='权限标识'),
    sa.PrimaryKeyConstraint('id')
    )
    op.create_table('role',
    sa.Column('id', sa.Integer(), nullable=False, comment='role id'),
    sa.Column('identifier', sa.String(length=64), nullable=False, comment='权限标识'),
    sa.Column('role_name', sa.String(length=32), nullable=False, comment='角色名称'),
    sa.Column('enabled', sa.SMALLINT(), nullable=True, comment='是否启用, 0 禁用 1 启用 2 废弃'),
    sa.Column('remark', sa.String(length=100), nullable=True, comment='备注'),
    sa.Column('created_at', sa.DateTime(), nullable=False, comment='创建时间'),
    sa.Column('updated_at', sa.DateTime(), nullable=False, comment='更新时间'),
    sa.PrimaryKeyConstraint('id')
    )
    with op.batch_alter_table('role', schema=None) as batch_op:
        batch_op.create_index(batch_op.f('ix_role_identifier'), ['identifier'], unique=False)

    op.create_table('user',
    sa.Column('id', sa.Integer(), nullable=False, comment='user id'),
    sa.Column('username', sa.String(length=64), nullable=True, comment='用户名'),
    sa.Column('phone', sa.String(length=11), nullable=True, comment='手机号'),
    sa.Column('email', sa.String(length=100), nullable=True, comment='邮箱'),
    sa.Column('password', sa.String(length=32), nullable=False, comment='密码'),
    sa.Column('status', sa.SMALLINT(), nullable=False, comment='状态 0 正常 1 禁用'),
    sa.Column('create_at', sa.DateTime(), nullable=False, comment='创建时间'),
    sa.Column('update_at', sa.DateTime(), nullable=False, comment='更新时间'),
    sa.PrimaryKeyConstraint('id')
    )
    op.create_table('user_token',
    sa.Column('user_id', sa.Integer(), nullable=False, comment='user token id'),
    sa.Column('token', sa.String(length=32), nullable=False, comment='token'),
    sa.Column('created_at', sa.DateTime(), nullable=False, comment='创建时间'),
    sa.Column('updated_at', sa.DateTime(), nullable=True, comment='更新时间'),
    sa.ForeignKeyConstraint(['user_id'], ['user.id'], ),
    sa.PrimaryKeyConstraint('user_id')
    )
    # ### end Alembic commands ###


def downgrade():
    # ### commands auto generated by Alembic - please adjust! ###
    op.drop_table('user_token')
    op.drop_table('user')
    with op.batch_alter_table('role', schema=None) as batch_op:
        batch_op.drop_index(batch_op.f('ix_role_identifier'))

    op.drop_table('role')
    op.drop_table('permission_role')
    with op.batch_alter_table('permission', schema=None) as batch_op:
        batch_op.drop_index(batch_op.f('ix_permission_identifier'))

    op.drop_table('permission')
    op.drop_table('db_user_role')
    # ### end Alembic commands ###