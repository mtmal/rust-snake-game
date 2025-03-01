use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Mutex;
use std::env;
use std::collections::HashMap;
use uuid::Uuid;
mod game;

/// Represents a player's score entry for the leaderboard
#[derive(Serialize, Deserialize)]
struct Score {
    name: String,
    score: u32,
}

/// Application state that is shared between all routes
/// Uses Mutex for thread-safe access to games and leaderboard
struct AppState {
    // Map of session IDs to game instances
    games: Mutex<HashMap<String, game::Game>>,
    // Vector of top scores
    leaderboard: Mutex<Vec<Score>>,
}

/// Serves the main HTML page
async fn index() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/index.html")?)
}

/// Returns the current state of a specific game
/// session_id: Unique identifier for the game instance
async fn get_game_state(
    session_id: web::Path<String>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let games = data.games.lock().unwrap();
    if let Some(game) = games.get(&session_id.into_inner()) {
        HttpResponse::Ok().json(game)
    } else {
        HttpResponse::NotFound().finish()
    }
}

/// Updates the direction of the snake for a specific game
/// session_id: Unique identifier for the game instance
/// direction: New direction for the snake
async fn update_direction(
    session_id: web::Path<String>,
    direction: web::Json<game::Direction>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let mut games = data.games.lock().unwrap();
    if let Some(game) = games.get_mut(&session_id.into_inner()) {
        game.direction = direction.into_inner();
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

/// Updates the game state (moves snake, checks collisions, etc.)
/// session_id: Unique identifier for the game instance
async fn update_game(
    session_id: web::Path<String>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let mut games = data.games.lock().unwrap();
    if let Some(game) = games.get_mut(&session_id.into_inner()) {
        game.update();
        HttpResponse::Ok().json(game)
    } else {
        HttpResponse::NotFound().finish()
    }
}

/// Makes an AI move for a specific game
/// session_id: Unique identifier for the game instance
async fn ai_move(
    session_id: web::Path<String>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let mut games = data.games.lock().unwrap();
    if let Some(game) = games.get_mut(&session_id.into_inner()) {
        game.ai_move();
        game.update();
        HttpResponse::Ok().json(game)
    } else {
        HttpResponse::NotFound().finish()
    }
}

/// Creates a new game instance and returns its session ID
async fn new_game(data: web::Data<AppState>) -> HttpResponse {
    // Generate a unique session ID
    let session_id = Uuid::new_v4().to_string();
    let mut games = data.games.lock().unwrap();
    // Create new game and store it in the HashMap
    games.insert(session_id.clone(), game::Game::new(20, 20));
    HttpResponse::Ok().json(json!({ "session_id": session_id }))
}

/// Submits a new score to the leaderboard
/// Keeps only top 10 scores
async fn submit_score(
    score: web::Json<Score>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let mut leaderboard = data.leaderboard.lock().unwrap();
    leaderboard.push(score.into_inner());
    // Sort leaderboard by score in descending order
    leaderboard.sort_by(|a, b| b.score.cmp(&a.score));
    // Keep only top 10 scores
    if leaderboard.len() > 10 {
        leaderboard.truncate(10);
    }
    HttpResponse::Ok().json(&*leaderboard)
}

/// Returns the current leaderboard
async fn get_leaderboard(data: web::Data<AppState>) -> HttpResponse {
    let leaderboard = data.leaderboard.lock().unwrap();
    HttpResponse::Ok().json(&*leaderboard)
}

/// Main function that sets up and runs the web server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Parse command line arguments for port number
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 {
        args[1].parse::<u16>().unwrap_or(8080)
    } else {
        8080
    };

    // Initialize shared application state
    let app_state = web::Data::new(AppState {
        games: Mutex::new(HashMap::new()),
        leaderboard: Mutex::new(Vec::new()),
    });

    println!("Starting server on port {}", port);
    
    // Configure and start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Serve static files from the 'static' directory
            .service(fs::Files::new("/static", "static").show_files_listing())
            // Define routes
            .route("/", web::get().to(index))
            .route("/game/{session_id}", web::get().to(get_game_state))
            .route("/direction/{session_id}", web::post().to(update_direction))
            .route("/update/{session_id}", web::post().to(update_game))
            .route("/ai-move/{session_id}", web::post().to(ai_move))
            .route("/new-game", web::post().to(new_game))
            .route("/submit-score", web::post().to(submit_score))
            .route("/leaderboard", web::get().to(get_leaderboard))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
} 