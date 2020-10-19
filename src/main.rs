use std::io;
use std::sync::Arc;

use actix_cors::Cors;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};

mod schema;

use crate::schema::{create_schema, Schema};

async fn graphiql() -> HttpResponse {
    let port = std::env::var("PORT").unwrap_or(String::from("8080"));

    let html = graphiql_source(&format!("http://0.0.0.0:{}/graphql", port));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let res = data.execute(&st, &());
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

async fn index_async(req: HttpRequest) -> &'static str {
    println!("REQ: {:?}", req);
    "Hello world!\r\n"
}

#[get("/")]
async fn no_params() -> &'static str {
    "Hello world!\r\n"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let port = std::env::var("PORT").unwrap_or(String::from("8080"));

    let schema = std::sync::Arc::new(create_schema());

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["POST", "GET"])
                    .supports_credentials()
                    .max_age(3600)
                    .finish(),
            )
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
            // .service(index)
            .service(no_params)
            .service(
                web::resource("/resource2/index.html")
                    .wrap(middleware::DefaultHeaders::new().header("X-Version-R2", "0.3"))
                    .default_service(web::route().to(|| HttpResponse::MethodNotAllowed()))
                    .route(web::get().to(index_async)),
            )
            .service(web::resource("/test1.html").to(|| async { "Test\r\n" }))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .workers(1)
    .run()
    .await
}
