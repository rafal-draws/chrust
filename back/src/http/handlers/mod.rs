use askama::Template;
use axum::response::{Html, IntoResponse};
use axum::http::StatusCode;

pub mod delete;
pub mod profile;
pub mod register;
pub mod upload;
pub mod track_menu;
pub mod help;



pub struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> axum::response::Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error {err}"),
            )
                .into_response(),
        }
    }
}