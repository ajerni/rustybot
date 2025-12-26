use llm_chain::{
    chains::sequential::Chain,
    options::{Opt, Options},
    prompt,
    step::Step,
    Parameters,
};
use llm_chain_openai::chatgpt::{Executor, Model};
use async_openai::config::OpenAIConfig;
use async_openai::Client;
use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult, middleware::Logger};
use actix_files::Files;
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use serde_json::json;

#[derive(Deserialize)]
struct CompletionRequest {
    question: String,
}

#[derive(Serialize)]
struct CompletionResponse {
    answer: String,
}

#[derive(Clone)]
struct AppState {
    executor: Arc<Executor>,
    chain: Arc<Chain>,
}

async fn completion(
    req: web::Json<CompletionRequest>,
    state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let parameters = Parameters::new()
        .with("question", &req.question);

    let output = state
        .chain
        .as_ref()
        .run(parameters, state.executor.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let answer = output
        .to_immediate()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .primary_textual_output()
        .unwrap_or_else(|| "No answer received".to_string());

    Ok(HttpResponse::Ok().json(CompletionResponse { answer }))
}

async fn greeter_with_name(nameparam: web::Path<String>) -> HttpResponse {
    let name = nameparam.as_str();
    HttpResponse::Ok().body(format!("Hello, {}!", name))
}

async fn greeter_default() -> HttpResponse {
    HttpResponse::Ok().body("Hello, world!")
}

async fn groqlive(req: web::Json<CompletionRequest>) -> ActixResult<HttpResponse> {
    let api_key = env::var("GROQ_API_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("GROQ_API_KEY not set"))?;

    let client = reqwest::Client::new();
    
    let request_body = json!({
        "model": "groq/compound-mini",
        "messages": [{
            "role": "user",
            "content": req.question
        }]
    });

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Request failed: {}", e)))?;

    let status = response.status();
    
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Ok(HttpResponse::build(status)
            .content_type("application/json")
            .json(json!({ "error": error_body })));
    }

    let groq_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse response: {}", e)))?;

    // Extract the answer from Groq's response format
    let answer = groq_response
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or("No answer received")
        .to_string();

    Ok(HttpResponse::Ok().json(CompletionResponse { answer }))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load .env if present
    dotenv::dotenv().ok();

    // -------------------------------------------------
    // 1️⃣  Build config pointing at OpenRouter
    // -------------------------------------------------
    let config = OpenAIConfig::new()
        .with_api_key(env::var("OPENROUTER_API_KEY")?)
        .with_api_base("https://openrouter.ai/api/v1");

    // set model from environment variable
    let model = env::var("MODEL").unwrap_or_else(|_| "meta-llama/llama-3.2-3b-instruct".to_string());
    // -------------------------------------------------
    // 2️⃣  Create executor with custom client and specific model
    // -------------------------------------------------
    let client = Client::with_config(config);
    let mut options_builder = Options::builder();
    options_builder.add_option(Opt::Model(Model::Other(model.to_string()).into()));
    let options = options_builder.build();
    let executor = Arc::new(Executor::for_client(client, options));

    // -------------------------------------------------
    // 3️⃣  Create prompt template - System prompt
    // -------------------------------------------------
    let step = Step::for_prompt_template(prompt!(
        "You are a helpful assistant. Answer concisely:\n{{question}}"
    ));

    // -------------------------------------------------
    // 4️⃣  Build the chain
    // -------------------------------------------------
    let chain = Chain::of_one(step);

    // -------------------------------------------------
    // 5️⃣  Set up Actix web server
    // -------------------------------------------------
    // Store executor and chain in AppState for efficient sharing.
    // Even though only /completion uses them, this avoids recreating
    // these expensive resources on every request.
    let app_state = AppState {
        executor: executor.clone(),
        chain: Arc::new(chain),
    };

    // Get port from environment variable (Render.com provides PORT)
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    
    let bind_address = format!("0.0.0.0:{}", port);

    println!("🚀 Starting server on http://0.0.0.0:{}", port);
    println!("📡 POST endpoint: http://0.0.0.0:{}/completion", port);
    println!("🌐 Static files served from /static directory");

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            .route("/completion", web::post().to(completion))
            .route("/groqlive", web::post().to(groqlive))
            // Register /name route BEFORE static files to avoid route conflicts
            .service(
                web::scope("/name")
                    .route("/{nameparam}", web::get().to(greeter_with_name))
                    .route("", web::get().to(greeter_default))
            )
            // Serve static files (images, etc.) from /static path
            .service(Files::new("/static", "./static"))
            // Serve index.html and other static files from root
            .service(
                Files::new("/", "./static")
                    .index_file("index.html")
            )
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}
