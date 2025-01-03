use std::fs::File;
use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use actix_web::middleware::Logger;
use actix_web::web::Query;
use csv::ReaderBuilder;
use reqwest::get;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, Level};
use tracing::level_filters::LevelFilter;
use tracing::log::warn;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use std::io::Cursor;
#[derive(Deserialize,Debug)]
struct QueryParams {
    id: Option<String>,
}

type CResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
async fn fetch_url(url: String, file_name: String) -> CResult<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(file_name)?;
    let mut content =  Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

#[get("/get")]
async fn get_info(query: Query<QueryParams>) -> HttpResponse {
    info!("{:#?}", &query);
    let url = "https://example.com/data.csv";  // Replace with your actual URL
    fetch_url("https://docs.google.com/spreadsheets/d/1xNPtI_fbzOx0KYQbnMwIL0v-4fImlDdTDMk19Y5Fy-c/gviz/tq?tqx=out:csv".to_string(), "data.csv".to_string()).await.unwrap();

    let file = std::fs::File::open("data.csv").unwrap();
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    if let Some(id) = query.id.clone() {
        let mut toret: Vec<Entry> = vec![];
        for result in rdr.deserialize() {
            let mut res: Entry = result.unwrap();
            let id = query.id.clone().unwrap().parse::<u32>().unwrap();
            if id == res.roll{
                toret.push(res.clone());
            }
        }
        HttpResponse::Ok().json(toret)
    }
    else {
        let x = rdr.deserialize().collect::<Result<Vec<Entry>,_>>().unwrap();;
        HttpResponse::Ok().json(x)

    }

}


#[derive(Debug, Deserialize,Clone,Serialize)]
pub struct Entry {
    #[serde(rename(deserialize = "ROLL"))]
    pub roll: u32,

    #[serde(rename(deserialize = "NAME"))]
    pub name: String,

    #[serde(rename(deserialize = "SUBJECT"))]
    pub subject: String,

    #[serde(rename(deserialize = "GROUP"))]
    pub group: String,

    #[serde(rename(deserialize = "DATE"))]
    pub date: String,

    #[serde(rename(deserialize = "SLOT"))]
    pub slot: String,

    #[serde(rename(deserialize = "DAY"))]
    pub day: String,
}


#[actix_web::main]
async fn main() {
    let does_dot_exist = dotenv::dotenv();
    match does_dot_exist {
        Ok(_) => info!(".env file found, using variables from .env"),
        Err(_) => warn!("env file not found"),
    }


    let debug_file =
        tracing_appender::rolling::hourly("./logs/", "debug").with_max_level(Level::TRACE);

    let warn_file =
        tracing_appender::rolling::hourly("./logs/", "warnings").with_max_level(Level::WARN);
    let all_files = debug_file.and(warn_file);

    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .from_env()
                .expect("Unable to read log level"),
        )
        .with(EnvFilter::from_env("LOG_LEVEL"))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(all_files)
                .with_ansi(false),
        )
        .with(
            tracing_subscriber::fmt::Layer::new()
                .with_ansi(true)
                .with_writer(std::io::stdout.with_max_level(Level::TRACE))
                // .with_file(true)
                .with_line_number(true),
        )
        .init();

    let _ = HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .service(get_info)
    })

        .bind(("0.0.0.0", 4444))
        .expect("Unable to start webserver")
        .run().await;
}