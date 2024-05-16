mod api;
mod auth;
mod appstate;
mod joke;
mod jokebase;
mod startup;
mod web;

use api::*;
use auth::*;
use appstate::*;
use joke::*;
use jokebase::*;
use startup::*;
use web::*;

use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use clap::Parser;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl, RedirectUrl, RevocationUrl};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
extern crate serde_json;
use sqlx::{
    self,
    postgres::{PgConnection, PgPool, PgRow, Postgres},
    Pool, Row,
};
extern crate thiserror;
use tokio::{self, sync::RwLock};
use tower_http::{services, trace};
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};
extern crate tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    openapi::schema::{ObjectBuilder, Schema, SchemaType},
    openapi::RefOr,
    OpenApi, ToSchema,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

const STYLESHEET: &str = "assets/static/knock-knock.css";

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Args {
    #[clap(short, long, default_value = "0.0.0.0:3000")]
    serve: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    startup(args.serve).await;
}
