use askama::Template;
use axum::{http::{HeaderValue, Response, StatusCode}, response::{Html, IntoResponse, Redirect}, Form};
use axum_extra::extract::CookieJar;
use reqwest::header::{LOCATION, SET_COOKIE};
use serde::Deserialize;

use crate::{db::db_conn::User, http::handlers::HtmlTemplate};



#[derive(Deserialize, Debug)]
    pub struct UserRegisterReq {
        pub username: String,
    }

    pub async fn register_user(Form(data): Form<UserRegisterReq>) -> impl IntoResponse {
        println!("extracted: {:?}", &data);

        let user = User::new(data.username).await;
        let cookie = format!("uuid={}; Path:/; HttpOnly; SameSite=Strict", &user.uuid);

        Response::builder()
            .status(StatusCode::FOUND)
            .header(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap())
            .header(LOCATION, "/profile")
            .body(axum::body::Body::empty())
            .unwrap()
    }



    pub async fn user_registered(jar: CookieJar) -> impl IntoResponse {
        if let Some(uuid) = jar.get("uuid") {
            Html(format!(
                "uuid: {}, <a href='/profile'>Go to profile</a>",
                uuid.value()
            ))
        } else {
            Html("No UUID found in cookies.".to_string())
        }
    }



    #[derive(Template)]
    #[template(path = "form.html")]
    pub struct UserFormTemplate {}


    pub async fn user_form(jar: CookieJar) -> Result<HtmlTemplate<UserFormTemplate>, Redirect> {
        if let Some(_uuid) = jar.get("uuid") {
            Err(Redirect::temporary("/profile"))
        } else {
            let template = UserFormTemplate {};
            Ok(HtmlTemplate(template))
        }
    }


    #[derive(Template)]
    #[template(path = "terms_of_use.html")]
    pub struct TermsOfUseTemplate {}

    pub async fn terms_of_use() -> HtmlTemplate<TermsOfUseTemplate> {
        HtmlTemplate(TermsOfUseTemplate {  })
    }
        
    