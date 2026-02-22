use proto::proto::core::{
    CreateCategoryRequest, DeleteCategoryRequest, SearchCategoryRequest, UpdateCategoryRequest,
};

use tracing::instrument;

use super::schema::{Category, CategoryMutation, NewCategory, SearchCategory};
use crate::resolvers::{
    convert,
    error::{Code, GqlError},
    utils::{connect_grpc_client, to_i64, to_option_i64},
};

#[instrument]
pub(crate) async fn create_category(category: NewCategory) -> Result<Vec<Category>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let name = category.name;

    let response = client
        .create_category(CreateCategoryRequest { name })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::category_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_category(search: SearchCategory) -> Result<Vec<Category>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .search_category(SearchCategoryRequest {
            name: search.name,
            category_id: to_option_i64(search.category_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::category_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_category(category_id: String) -> Result<Vec<Category>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let category_id = category_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse category id", Code::InvalidArgument))?;

    let response = client
        .delete_category(DeleteCategoryRequest { category_id })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::category_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_category(category: CategoryMutation) -> Result<Vec<Category>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .update_category(UpdateCategoryRequest {
            name: category.name,
            category_id: to_i64(category.category_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::category_response_to_gql)
        .collect())
}
