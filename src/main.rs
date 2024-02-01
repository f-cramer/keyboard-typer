use std::env;
use std::mem::size_of;
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::Duration;

use axum::{Json, Router, serve};
use axum::routing::post;
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_UNICODE, SendInput,
};

const INPUT_SIZE: i32 = size_of::<INPUT>() as i32;

const INPUT_DELAY_DEFAULT: Duration = Duration::from_secs(1);
const CHARACTER_DELAY_DEFAULT: Duration = Duration::from_millis(50);

static INPUT_DELAY: OnceLock<Duration> = OnceLock::new();
static CHARACTER_DELAY: OnceLock<Duration> = OnceLock::new();

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let input_delay = parse_duration(&args, "--input-delay=").unwrap_or(INPUT_DELAY_DEFAULT);
    let character_delay = parse_duration(&args, "--character-delay=").unwrap_or(CHARACTER_DELAY_DEFAULT);

    INPUT_DELAY.set(input_delay).unwrap();
    CHARACTER_DELAY.set(character_delay).unwrap();

    println!("using default input_delay = {:?}", INPUT_DELAY.get().unwrap());
    println!("using default character_delay = {:?}", CHARACTER_DELAY.get().unwrap());

    start_server().await?;
    Ok(())
}

fn parse_arg<'a>(args: &'a Vec<String>, prefix: &str) -> Option<&'a str> {
    args.iter().find(|e| e.starts_with(prefix)).map(|e| &e[prefix.len()..])
}

fn parse_duration(args: &Vec<String>, prefix: &str) -> Option<Duration> {
    parse_arg(args, prefix).and_then(|e| e.parse::<u64>().ok()).map(|e| Duration::from_millis(e))
}

async fn start_server() -> std::io::Result<(), > {
    let app = Router::new().route("/", post(root_handler))
        .layer(CorsLayer::permissive());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("listening on {}", listener.local_addr().unwrap());
    serve(listener, app).await?;
    Ok(())
}

async fn root_handler(Json(body): Json<RequestBody>) {
    println!("received request containing body: {:?}", body);
    tokio::time::sleep(body.delay.map(|d| Duration::from_millis(d)).unwrap_or_else(|| *INPUT_DELAY.get().unwrap())).await;
    send_input(body.text.as_str(), body.per_key_delay.map(|d| Duration::from_millis(d)).unwrap_or_else(|| *CHARACTER_DELAY.get().unwrap()));
}

#[derive(Clone, Debug, Deserialize)]
struct RequestBody {
    text: String,
    delay: Option<u64>,
    per_key_delay: Option<u64>,
}

fn send_input(s: &str, per_key_delay: Duration) {
    let mut first = true;
    for c in s.chars() {
        if first {
            first = false;
        } else {
            sleep(per_key_delay);
        }

        send_input_char(c);
    }
}

fn send_input_char(c: char) {
    let input = char_to_input(c);
    unsafe {
        SendInput(1, &input, INPUT_SIZE);
    }
}

fn char_to_input(c: char) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: c as u16,
                dwFlags: KEYEVENTF_UNICODE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
