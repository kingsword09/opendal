use http::{header, Request, StatusCode};
use serde_json::Value;

use super::error::parse_error;
use super::model::D1Response;
use crate::raw::*;
use crate::*;

#[derive(Debug, Clone)]
pub struct D1Core {
    pub authorization: Option<String>,
    pub account_id: String,
    pub database_id: String,
    pub client: HttpClient,
    pub table: String,
    pub key_field: String,
    pub value_field: String,
}

impl D1Core {
    pub fn create_d1_query_request(
        &self,
        sql: &str,
        params: Vec<Value>,
    ) -> Result<Request<Buffer>> {
        let p = format!(
            "/accounts/{}/d1/database/{}/query",
            self.account_id, self.database_id
        );
        let url: String = format!(
            "{}{}",
            "https://api.cloudflare.com/client/v4",
            percent_encode_path(&p)
        );

        let mut req = Request::post(&url);
        if let Some(auth) = &self.authorization {
            req = req.header(header::AUTHORIZATION, auth);
        }
        req = req.header(header::CONTENT_TYPE, "application/json");

        let json = serde_json::json!({
            "sql": sql,
            "params": params,
        });

        let body = serde_json::to_vec(&json).map_err(new_json_serialize_error)?;
        req.body(Buffer::from(body))
            .map_err(new_request_build_error)
    }

    pub async fn get(&self, path: &str) -> Result<Option<Buffer>> {
        let query = format!(
            "SELECT {} FROM {} WHERE {} = ? LIMIT 1",
            self.value_field, self.table, self.key_field
        );
        let req = self.create_d1_query_request(&query, vec![path.into()])?;

        let resp = self.client.send(req).await?;
        let status = resp.status();
        match status {
            StatusCode::OK | StatusCode::PARTIAL_CONTENT => {
                let body = resp.into_body();
                let bs = body.to_bytes();
                let d1_response = D1Response::parse(&bs)?;
                Ok(d1_response.get_result(&self.value_field))
            }
            _ => Err(parse_error(resp)),
        }
    }

    pub async fn get_range(
        &self,
        path: &str,
        start: isize,
        limit: isize,
    ) -> Result<Option<Buffer>> {
        let query = format!(
            "SELECT {} FROM {} WHERE {} = ? LIMIT {} OFFSET {}",
            self.value_field, self.table, self.key_field, limit, start
        );
        let req = self.create_d1_query_request(&query, vec![path.into()])?;

        let resp = self.client.send(req).await?;
        let status = resp.status();
        match status {
            StatusCode::OK | StatusCode::PARTIAL_CONTENT => {
                let body = resp.into_body();
                let bs = body.to_bytes();
                let d1_response = D1Response::parse(&bs)?;
                Ok(d1_response.get_result(&self.value_field))
            }
            _ => Err(parse_error(resp)),
        }
    }

    pub async fn set(&self, path: &str, value: Buffer) -> Result<()> {
        let table = &self.table;
        let key_field = &self.key_field;
        let value_field = &self.value_field;
        let query = format!(
            "INSERT INTO {table} ({key_field}, {value_field}) \
                VALUES (?, ?) \
                ON CONFLICT ({key_field}) \
                    DO UPDATE SET {value_field} = EXCLUDED.{value_field}",
        );

        let params = vec![path.into(), value.to_vec().into()];
        let req = self.create_d1_query_request(&query, params)?;

        let resp = self.client.send(req).await?;
        let status = resp.status();
        match status {
            StatusCode::OK | StatusCode::PARTIAL_CONTENT => Ok(()),
            _ => Err(parse_error(resp)),
        }
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let query = format!("DELETE FROM {} WHERE {} = ?", self.table, self.key_field);
        let req = self.create_d1_query_request(&query, vec![path.into()])?;

        let resp = self.client.send(req).await?;
        let status = resp.status();
        match status {
            StatusCode::OK | StatusCode::PARTIAL_CONTENT => Ok(()),
            _ => Err(parse_error(resp)),
        }
    }
}
