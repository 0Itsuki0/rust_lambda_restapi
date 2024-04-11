use crate::event::Event;
use crate::handler_params::QueryParams;

use anyhow::{bail, Ok, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use axum::response::Json;
// use serde_dynamo::aws_sdk_dynamodb_1::to_item;
use serde_dynamo::{from_item, from_items, to_item};
use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct EventService {
    db_client: Client,
    table_name: String,
}

// public impl
impl EventService {
    pub fn new(db_client: Client, table_name: &str) -> Self {
        Self {
            db_client,
            table_name: table_name.to_owned(),
        }
    }

    pub async fn get_events(&self, params: QueryParams) -> Result<Json<Value>> {
        println!("getting events with paras {:?}", params);

        let mut builder = self.db_client.scan().table_name(&self.table_name);
        if let Some(title) = params.title.clone() {
            builder = builder
                .filter_expression("#name = :value")
                .expression_attribute_names("#name", "title")
                .expression_attribute_values(":value", AttributeValue::S((&title).clone()));
        }

        let mut results = builder.clone().send().await?;

        if let Some(items) = results.items {
            let mut events: Vec<Event> = from_items(items)?;

            while let Some(last_evaluated_key) = &results.last_evaluated_key {
                results = builder
                    .clone()
                    .set_exclusive_start_key(Option::Some(last_evaluated_key.to_owned()))
                    .send()
                    .await?;
                if let Some(new_items) = results.items {
                    let mut new_events: Vec<Event> = from_items(new_items)?;
                    events.append(&mut new_events);
                } else {
                    break;
                }
            }

            return Ok(Json(json!({
                "error": false,
                "events": events
            })));
        } else {
            return Ok(Json(json!({})));
        }
    }

    pub async fn post_event(&self, event: Event) -> Result<Json<Value>> {
        if self.event_exist(&event.id).await? {
            bail!("Event exists!")
        }
        let builder = self
            .db_client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(to_item(&event)?));

        builder.send().await?;

        Ok(Json(json!({
            "error": false,
            "message": "event added."
        })))
    }

    pub async fn get_event_single(&self, id: String) -> Result<Json<Value>> {
        let results = self
            .db_client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#name = :value")
            .expression_attribute_names("#name", "id")
            .expression_attribute_values(":value", AttributeValue::S(id.to_owned()))
            .send()
            .await?;
        if results.count == 0
            || results.items.is_none()
            || results.items.clone().unwrap().is_empty()
        {
            bail!("Event does not exist for id:{id}!")
        }
        let item = results.items.unwrap().first().unwrap().to_owned();
        let event: Event = from_item(item)?;
        Ok(Json(json!({
            "error": false,
            "event": event
        })))
    }

    pub async fn delete_event_single(&self, id: String) -> Result<Json<Value>> {
        if !self.event_exist(&id).await? {
            bail!("Event does not exist for id: {id}!")
        }
        self.db_client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.clone()))
            .send()
            .await?;

        Ok(Json(json!({
            "error": false,
            "message": "event for id: ".to_owned() + &id + " deleted."
        })))
    }

    pub async fn put_event_title(&self, id: String, title: String) -> Result<Json<Value>> {
        if !self.event_exist(&id).await? {
            bail!("Event does not exist for id: {id}!")
        }

        self.db_client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.clone()))
            .update_expression("set #name = :value")
            .expression_attribute_names("#name", "title")
            .expression_attribute_values(":value", AttributeValue::S(title.to_owned()))
            .send()
            .await?;

        Ok(Json(json!({
            "error": false,
            "event": "Event title for id: ".to_owned() + &id + " changed to " + &title
        })))
    }
}

// private impl
impl EventService {
    async fn event_exist(&self, id: &str) -> Result<bool> {
        let results = self
            .db_client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#name = :value")
            .expression_attribute_names("#name", "id")
            .expression_attribute_values(":value", AttributeValue::S(id.to_owned()))
            .send()
            .await?;

        Ok(results.count > 0)
    }
}
