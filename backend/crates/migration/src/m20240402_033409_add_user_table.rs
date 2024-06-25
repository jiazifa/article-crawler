use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("ssr.customer"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .auto_increment()
                            .primary_key()
                            .not_null()
                            .comment("主键".to_string()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("nick_name"))
                            .string_len(100)
                            .not_null()
                            .comment("昵称".to_string()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("email"))
                            .string_len(100)
                            .not_null()
                            .comment("邮箱".to_string()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("password"))
                            .string_len(40)
                            .comment("密码".to_string()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("avatar"))
                            .string_len(200)
                            .comment("头像".to_string()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("birth"))
                            .date()
                            .comment("出生日期".to_string()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("gender"))
                            .small_integer()
                            .comment("性别 1 男 2 女".to_string()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("last_login_time"))
                            .date_time()
                            .comment("上一次登录时间".to_string()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("create_time"))
                            .default(Expr::current_timestamp())
                            .date_time()
                            .comment("创建时间（注册时间）".to_string()),
                    )
                    .col(
                        ColumnDef::new(Alias::new("update_time"))
                            .default(Expr::current_timestamp())
                            .date_time()
                            .comment("更新时间".to_string()),
                    )
                    .comment("用户表".to_string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("ssr.customer"))
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
