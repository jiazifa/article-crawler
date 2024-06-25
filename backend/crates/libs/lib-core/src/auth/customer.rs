use lib_entity::rss_customer;

use crate::{error::ErrorInService, DBConnection};

use super::schema::{CustomerModel, QueryCustomerByIDRequest};
use sea_orm::{entity::*, query::*};

impl QueryCustomerByIDRequest {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

pub struct CustomerController;

impl CustomerController {
    pub async fn query_customer_by_id(
        &self,
        req: QueryCustomerByIDRequest,
        conn: &DBConnection,
    ) -> Result<Option<CustomerModel>, ErrorInService> {
        let model = rss_customer::Entity::find()
            .filter(rss_customer::Column::Id.eq(req.id))
            .into_model()
            .one(conn)
            .await
            .map_err(ErrorInService::DBError)?;

        Ok(model)
    }
}
