// 创建一张表
use anyhow::Result;
pub use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, Schema};
use sea_orm_migration::{
    prelude::*,
    sea_orm::{DatabaseBackend, EntityTrait, IdenStatic},
};

/// 创建表格
///
/// e Entity
pub async fn create_one_table<E>(
    db: &dyn ConnectionTrait,
    builder: DatabaseBackend,
    schema: &Schema,
    e: E,
) -> Result<(), DbErr>
where
    E: EntityTrait,
{
    match db
        .execute(
            builder.build(
                schema
                    .create_table_from_entity(e)
                    .to_owned()
                    .if_not_exists(),
            ),
        )
        .await
    {
        Ok(_) => println!("创建表格成功:{}", e.table_name()),
        Err(e) => println!("创建表格失败:{}", e),
    };

    Ok(())
}

///  创建表格索引
///
///  t:表格主题 `Entity`;
/// name:索引名称;
/// tp:索引类型 => u:unique,p:primary,i:index,f:fulltext;
pub async fn create_table_index<C, T>(
    manager: &SchemaManager<'_>,
    t: T,
    name: &str,
    cols: Vec<C>,
    tp: &str,
) -> Result<(), DbErr>
where
    C: 'static + IntoIndexColumn + IdenStatic + Clone + Copy,
    T: 'static + Iden + EntityTrait,
{
    let mut index = Index::create().name(name).table(t).to_owned();
    let mut cols_name = Vec::<String>::new();
    for co in cols {
        index = index.col(co).to_owned();
        cols_name.push(co.as_str().to_string());
    }
    match tp {
        "u" => {
            index = index.unique().to_owned();
        }
        "p" => {
            index = index.primary().to_owned();
        }
        "f" => {
            index = index.full_text().to_owned();
        }
        _ => {}
    }
    match manager.create_index(index.to_owned()).await {
        Ok(_) => println!(
            "成功创建索引,表格:{},索引名:{},索引列:{:?}",
            t.table_name(),
            name,
            cols_name
        ),
        Err(e) => println!("{}", e),
    };

    Ok(())
}

///  删除一张表
pub async fn drop_one_table<T>(manager: &SchemaManager<'_>, t: T) -> Result<(), DbErr>
where
    T: EntityTrait + IntoTableRef + 'static,
{
    manager
        .drop_table(Table::drop().table(t).to_owned())
        .await?;
    println!("成功删除表格:{}", t.table_name());
    Ok(())
}
