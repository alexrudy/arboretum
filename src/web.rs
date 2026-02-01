use std::{
    convert::Infallible,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use axum::http;
use camino::Utf8Path;
use chrono::TimeZone;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web/build"]
pub struct Assets;

#[derive(Debug)]
pub struct EmbedServer<E> {
    embed: PhantomData<E>,
}

impl<E> Clone for EmbedServer<E> {
    fn clone(&self) -> Self {
        Self { embed: PhantomData }
    }
}

impl<E> Default for EmbedServer<E>
where
    E: Embed,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<E> EmbedServer<E>
where
    E: Embed,
{
    pub fn new() -> Self {
        Self { embed: PhantomData }
    }

    pub fn serve<B>(&self, req: http::Request<B>) -> http::Response<axum::body::Body> {
        let mut response = http::Response::builder();

        match req.method() {
            &http::Method::GET | &http::Method::HEAD => {}
            _ => {
                return response
                    .header(http::header::ALLOW, "GET, HEAD, OPTIONS")
                    .status(http::StatusCode::METHOD_NOT_ALLOWED)
                    .body(axum::body::Body::empty())
                    .unwrap();
            }
        }

        let path = req.uri().path().trim_start_matches('/');

        let mut request_file_path: String = path.into();

        // If there is no extension, add html
        // If the path is empty, default to index.html
        let p = Utf8Path::new(&request_file_path);
        if p.file_name().is_some() && p.extension().is_none() {
            request_file_path.push_str(".html");
        } else if p.file_name().is_none() {
            request_file_path.push_str("index.html");
        }

        let content_type = mime_guess::from_path(&request_file_path).first_or_octet_stream();

        match E::get(&request_file_path) {
            Some(file) => {
                response = response.header(http::header::CONTENT_TYPE, content_type.to_string());
                response = response.header(http::header::CONTENT_LENGTH, file.data.len());

                let etag = data_encoding::BASE64URL_NOPAD.encode(&file.metadata.sha256_hash());
                response = response
                    .status(http::StatusCode::OK)
                    .header(http::header::ETAG, &etag);
                response =
                    response.header(http::header::CACHE_CONTROL, "max-age=0, must-revalidate");

                if let Some(last_modified) = file
                    .metadata
                    .last_modified()
                    .and_then(|ts| i64::try_from(ts).ok())
                    .and_then(|ts| chrono::Utc.timestamp_opt(ts, 0).earliest())
                {
                    response = response.header(
                        http::header::LAST_MODIFIED,
                        last_modified.with_timezone(&chrono_tz::GMT).to_rfc2822(),
                    );
                }

                match req
                    .headers()
                    .get(http::header::IF_NONE_MATCH)
                    .and_then(|h| h.to_str().ok())
                {
                    Some(header_etag) if etag.eq(header_etag) => response
                        .status(http::StatusCode::NOT_MODIFIED)
                        .body(axum::body::Body::empty())
                        .unwrap(),
                    _ if req.method() == http::Method::HEAD => {
                        response.body(axum::body::Body::empty()).unwrap()
                    }
                    _ => response.body(axum::body::Body::from(file.data)).unwrap(),
                }
            }
            None => http::Response::builder()
                .status(http::StatusCode::NOT_FOUND)
                .body(axum::body::Body::empty())
                .unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct EmbedFuture(Option<Result<http::Response<axum::body::Body>, Infallible>>);

impl Future for EmbedFuture {
    type Output = Result<http::Response<axum::body::Body>, Infallible>;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.0.take().expect("future polled after ready"))
    }
}

impl<B, E> tower::Service<http::Request<B>> for EmbedServer<E>
where
    E: Embed,
{
    type Response = http::Response<axum::body::Body>;

    type Error = Infallible;

    type Future = EmbedFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        EmbedFuture(Some(Ok(self.serve(req))))
    }
}
