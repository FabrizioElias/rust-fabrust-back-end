use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router, extract::Path,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use chrono::{DateTime, Utc};
use mongodb::{bson::{doc, Bson, self, DateTime as BsonDateTime}, options::{ClientOptions, ServerApi, ServerApiVersion}, Client, Collection};

#[tokio::main]
async fn main() {
    //cargo watch -c -q -x run
    // initialize tracing
    //tracing_subscriber::fmt::init();
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods(Any)
        .allow_headers(Any)
        // allow requests from any origin
        .allow_origin(Any);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/weight/measurement/:id", get(get_weight_measurement_id))
        // `POST /users` goes to `create_user`
        .route("/weight/measurement", post(create_weight_measurement))
        .layer(cors);

    env::set_var("mongoDb.connectionString", "mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000");
    
    tracing::info!("listening on {}", "127.0.0.1:3000");
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_collection<T>(database: &str, collection: &str) -> mongodb::error::Result<Collection<T>> {
    let mongodb_conn_string = read_env_var("mongoDb.connectionString", "localhost:4666");

    tracing::info!("DB on {}", mongodb_conn_string);
    let mut client_options = ClientOptions::parse(mongodb_conn_string).await?;
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;
    // Get the database and colletion handles
    let collection = client.database(database).collection::<T>(collection);

    Ok(collection)
}

fn read_env_var(env_name: &str, default: &str) -> String {
    match env::var(env_name) {
        Ok(v) => v,
        Err(_e) => default.to_string()
    }
}

// basic handler that responds with a static string
async fn get_weight_measurement_id(Path(id): Path<String>) -> impl IntoResponse {
    let collection = match get_collection::<WheightMeasurementEntity>("fabdev", "Wheights").await {
        Ok(c) => c,
        Err(e) => panic!("Error getting collection: {}", e)
    };

    let filter = doc! { "_id": bson::oid::ObjectId::parse_str(id).unwrap() };
    tracing::info!("Filter: {:?}", filter);
    let result = match collection.find_one(filter, None).await {
        Ok(r) => r,
        Err(e) => panic!("Error finding document: {}", e)
    };

    match result {
        Some(r) => (StatusCode::OK, Json(WheightMeasurementOutput::from_entity(r))).into_response(),
        None => (StatusCode::NOT_FOUND, Json(ErrorResponse { message: "Not Found" })).into_response()   
    }
}

async fn create_weight_measurement(
    // this argument tells axum to parse the request body
    // as JSON into a `WheightMeasurementInput` type
    Json(payload): Json<WheightMeasurementInput>,
) -> impl IntoResponse {
    // insert your application logic here
    let measurement = WheightMeasurementEntity {
        _id: bson::oid::ObjectId::default(),
        date: BsonDateTime::from_chrono(payload.date),
        wheight_kg: payload.wheight_kg,
        imc: payload.imc,
        fat_percentage: payload.fat_percentage,
        water_percentage: payload.water_percentage,
        protein_percentage: payload.protein_percentage,
        metabolism_kcal: payload.metabolism_kcal,
        visceral_fat_index: payload.visceral_fat_index,
        muscle_kg: payload.muscle_kg,
        bone_kg: payload.bone_kg,
        metabolic_age: payload.metabolic_age,
        fat_kg: payload.wheight_kg * (payload.fat_percentage / 100_f32),
        muscle_percentage: 100_f32 * (payload.muscle_kg / payload.wheight_kg)
    };

    let collection = match get_collection::<WheightMeasurementEntity>("fabdev", "Wheights").await {
        Ok(c) => c,
        Err(e) => panic!("Error getting collection: {}", e)
    };

    let result = match collection.insert_one(measurement, None).await{
        Ok(r) => r,
        Err(e) => panic!("Error inserting document: {}", e)
    };

    let response = WheightMeasurementIdResponse { id: result.inserted_id.to_string() };
    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(response))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct WheightMeasurementInput {
    date: DateTime<Utc>,
    wheight_kg: f32,
    imc: f32,
    fat_percentage: f32,
    water_percentage: f32,
    protein_percentage: f32,
    metabolism_kcal: f32,
    visceral_fat_index: f32,
    muscle_kg: f32,
    bone_kg: f32,
    metabolic_age: u8
}

#[derive(Serialize)]
struct WheightMeasurementIdResponse {
    id: String
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    message: &'a str
}

#[derive(Serialize, Deserialize)]
struct WheightMeasurementEntity {
    _id: bson::oid::ObjectId,
    date: mongodb::bson::DateTime,
    wheight_kg: f32,
    imc: f32,
    fat_percentage: f32,
    water_percentage: f32,
    protein_percentage: f32,
    metabolism_kcal: f32,
    visceral_fat_index: f32,
    muscle_kg: f32,
    bone_kg: f32,
    metabolic_age: u8,
    fat_kg: f32,
    muscle_percentage: f32
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct WheightMeasurementOutput {
    id: String,
    date: DateTime<Utc>,
    wheight_kg: f32,
    imc: f32,
    fat_percentage: f32,
    water_percentage: f32,
    protein_percentage: f32,
    metabolism_kcal: f32,
    visceral_fat_index: f32,
    muscle_kg: f32,
    bone_kg: f32,
    metabolic_age: u8,
    fat_kg: f32,
    muscle_percentage: f32,
    wheight_kg_diff: f32,
    fat_percentage_diff: f32,
    muscle_kg_diff: f32,
    bone_kg_diff: f32,
    fat_kg_diff: f32,
    muscle_percentage_diff: f32
}

impl WheightMeasurementOutput {
    pub fn from_entity(entity: WheightMeasurementEntity) -> Self {
        WheightMeasurementOutput {
            id: entity._id.to_string(),
            date: entity.date.to_chrono(),
            wheight_kg: entity.wheight_kg,
            imc: entity.imc,
            fat_percentage: entity.fat_percentage,
            water_percentage: entity.water_percentage,
            protein_percentage: entity.protein_percentage,
            metabolism_kcal: entity.metabolism_kcal,
            visceral_fat_index: entity.visceral_fat_index,
            muscle_kg: entity.muscle_kg,
            bone_kg: entity.bone_kg,
            metabolic_age: entity.metabolic_age,
            fat_kg: entity.fat_kg,
            muscle_percentage: entity.muscle_percentage,
            wheight_kg_diff: 0_f32,
            fat_percentage_diff: 0_f32,
            muscle_kg_diff: 0_f32,
            bone_kg_diff: 0_f32,
            fat_kg_diff: 0_f32,
            muscle_percentage_diff: 0_f32
        }
    }
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct WheightMeasurementMovingAvarageOutput {
    id: u64,
    date: DateTime<Utc>,
    wheight_kg: f32,
    fat_kg: f32,
    muscle_kg: f32,
    muscle_percentage: f32,
    fat_percentage: f32,
    wheight_kg_diff: f32,
    fat_kg_diff: f32,
    muscle_kg_diff: f32,
    muscle_percentage_diff: f32,
    fat_percentage_diff: f32
}