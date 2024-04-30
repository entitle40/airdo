use serde::{Deserialize, Serialize};
use sqlx::{Execute, Pool, QueryBuilder, Sqlite};

use crate::util::response_util::PageInfo;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, sqlx::FromRow)]
pub struct HealthCheck {
    pub id: Option<u32>,
    pub create_time: Option<i64>,
    pub request_time: Option<i64>,
    pub node_name: Option<String>,
    pub status_code: Option<u16>,
    pub status_des: Option<String>,
    pub delay_ms: Option<i32>,
}

pub async fn create(entity: HealthCheck, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("insert into health_check(");
    let mut separated = query_builder.separated(", ");
    if entity.create_time.is_some() {
        separated.push("create_time");
    }
    if entity.request_time.is_some() {
        separated.push("request_time");
    }
    if entity.node_name.is_some() {
        separated.push("node_name");
    }
    if entity.status_code.is_some() {
        separated.push("status_code");
    }
    if entity.status_des.is_some() {
        separated.push("status_des");
    }
    if entity.delay_ms.is_some() {
        separated.push("delay_ms");
    }
    query_builder.push(")  values(");
    let mut separated = query_builder.separated(", ");
    if entity.create_time.is_some() {
        separated.push_bind(entity.create_time.unwrap());
    }
    if entity.request_time.is_some() {
        separated.push_bind(entity.request_time.unwrap());
    }
    if entity.node_name.is_some() {
        separated.push_bind(entity.node_name.unwrap());
    }
    if entity.status_code.is_some() {
        separated.push_bind(entity.status_code.unwrap());
    }
    if entity.status_des.is_some() {
        separated.push_bind(entity.status_des.unwrap());
    }
    if entity.delay_ms.is_some() {
        separated.push_bind(entity.delay_ms.unwrap());
    }
    query_builder.push(")");

    let query = query_builder.build();
    tracing::debug!("插入健康检查SQL：{}", query.sql());
    let res = query.execute(pool).await;
    tracing::debug!("插入健康检查结果：{:?}", res);
    res
}

#[allow(dead_code)]
pub async fn page(start_time: chrono::DateTime<chrono::Local>, end_time: chrono::DateTime<chrono::Local>, page_number: u32, page_size: u32, pool: &Pool<Sqlite>) -> Result<PageInfo<HealthCheck>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select count(*) from health_check");
    query_builder.push(" where request_time >= DATETIME(").push_bind(start_time).push(")");
    query_builder.push(" and request_time < DATETIME(").push_bind(end_time).push(")");
    let query = query_builder.build_query_as::<(i64,)>();
    tracing::debug!("查询健康检查SQL：{}", query.sql());
    let res = query.fetch_one(pool).await;
    tracing::debug!("查询健康检查结果：{:?}", res);
    let count: (i64,) = res?;
    if count.0 <= 0 {
        return Ok(PageInfo::new(0, vec![]));
    }

    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from health_check");
    query_builder.push(" where request_time >= DATETIME(").push_bind(start_time).push(")");
    query_builder.push(" and request_time < DATETIME(").push_bind(end_time).push(")");
    query_builder.push(" order by id desc");
    query_builder.push(" limit").push_bind((page_number - 1) * page_size).push(",").push_bind(page_size);
    let query = query_builder.build_query_as::<HealthCheck>();
    tracing::debug!("查询健康检查SQL：{}", query.sql());
    let res = query.fetch_all(pool).await;
    tracing::debug!("查询健康检查结果：{:?}", res);
    Ok(PageInfo::new(count.0 as u32, res?))
}

pub async fn list(start_time: i64, end_time: i64, pool: &Pool<Sqlite>) -> Result<Vec<HealthCheck>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("select * from health_check");
    query_builder.push(" where request_time >= ").push_bind(start_time);
    query_builder.push(" and request_time < ").push_bind(end_time);
    query_builder.push(" order by request_time asc");
    let query = query_builder.build_query_as::<HealthCheck>();
    tracing::debug!("查询健康检查SQL：{}", query.sql());
    let res = query.fetch_all(pool).await;
    tracing::debug!("查询健康检查结果：{:?}", res);
    Ok(res?)
}

pub async fn delete_before_time(days: i64, pool: &Pool<Sqlite>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let days_ago = chrono::Local::now() - chrono::Duration::try_days(days).unwrap();
    let mut query_builder = QueryBuilder::new("delete from proxy_node where request_time < ");
    query_builder.push_bind(days_ago.format("%Y-%m-%d %H:%M:%S").to_string());
    let query = query_builder.build();
    tracing::debug!("删除健康检查SQL：{}", query.sql());
    let res = query.execute(pool).await;
    tracing::debug!("删除健康检查结果：{:?}", res);
    res
}
