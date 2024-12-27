use chrono::{Duration, Local};
use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};
use std::{fmt::Display, str::FromStr};

pub fn use_cookie<T>(name: &str) -> (Signal<Option<T>>, WriteSignal<Option<T>>)
where
    T: Send + Sync + FromStr + Display + Clone + 'static,
{
    let expires_at = (Local::now() + Duration::days(365)).timestamp();
    let cookie_options = UseCookieOptions::default()
        .path("/")
        .expires(expires_at)
        .same_site(SameSite::Strict);
    use_cookie_with_options::<T, FromToStringCodec>(name, cookie_options)
}
