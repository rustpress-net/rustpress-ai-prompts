//! RSS Feed Handler

use crate::models::*;
use crate::services::ServiceError;
use crate::BlogServices;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use rss::{ChannelBuilder, ItemBuilder};
use std::sync::Arc;

/// GET /feed - RSS feed
pub async fn rss_feed(
    State(services): State<Arc<BlogServices>>,
) -> Result<impl IntoResponse, ServiceError> {
    let query = PostQuery {
        page: Some(1),
        per_page: Some(20),
        category: None,
        tag: None,
        author: None,
        status: Some(PostStatus::Published),
        sort: Some("date".into()),
        order: Some("desc".into()),
    };

    let posts = services.posts.list_published(&query).await?;

    let items: Vec<_> = posts
        .data
        .iter()
        .map(|post| {
            ItemBuilder::default()
                .title(Some(post.post.title.clone()))
                .link(Some(format!("/posts/{}", post.post.slug)))
                .description(post.post.excerpt.clone())
                .author(Some(post.author.name.clone()))
                .pub_date(post.post.published_at.map(|d| d.to_rfc2822()))
                .build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title("Blog")
        .link("/")
        .description("Latest blog posts")
        .items(items)
        .build();

    let xml = channel.to_string();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")
        .body(xml)
        .unwrap())
}
