use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use serde::{Deserialize, Serialize};

use std::fs::OpenOptions;
use std::io::{BufReader, Seek, SeekFrom, Write};

type Token = String;

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    uuid: String,
    msg: Message,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    payload: String,
}

enum ResponseCode {
    InternalServerError,
    BadRequest,
    Unauthorized,
}

fn generate_token() -> Token {
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let token_string: Token = rng.sample_iter(&Alphanumeric).take(7).collect();
    token_string
}

fn save_token_to_file(token: &str) -> Result<(), std::io::Error> {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("tokens.json");
    file.and_then(|mut f| {
        let reader = BufReader::new(&f);
        let tokens = serde_json::from_reader(reader);
        // if 'tokens.json' is empty or contains invalid data return an empty list
        let mut tokens: Vec<Token> = tokens.unwrap_or_default();
        tokens.push(token.to_string());
        // overwrite the previous contents
        f.seek(SeekFrom::Start(0))?;
        f.write_all(serde_json::to_string(&tokens).unwrap().as_bytes())
    })
}

async fn authorize(token: Token) -> Result<(), ResponseCode> {
    let tokens: Option<Vec<Token>> = (|| {
        let tokens_file = OpenOptions::new().read(true).open("tokens.json").ok()?;
        let reader = BufReader::new(tokens_file);
        serde_json::from_reader(reader).ok()
    })();

    if tokens.is_none() {
        return Err(ResponseCode::InternalServerError);
    }
    let tokens: Vec<Token> = tokens.unwrap();
    if !tokens.contains(&token) {
        return Err(ResponseCode::Unauthorized);
    }
    Ok(())
}

fn save_message_to_file(message: Message) -> Result<(), std::io::Error> {
    let mut messages_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("messages.json")?;
    let messages_reader = BufReader::new(&messages_file);
    let messages = serde_json::from_reader(messages_reader);
    // if 'messages.json' is empty or contains invalid data return an empty list
    let mut messages: Vec<Message> = messages.unwrap_or_default();
    messages.push(message);
    // overwrite the previous contents
    messages_file.seek(SeekFrom::Start(0))?;
    let messages_string = serde_json::to_string(&messages).unwrap();
    let serialized_messages = messages_string.as_bytes();
    messages_file.write_all(serialized_messages)?;
    Ok(())
}

async fn handle_message(body: Body) -> Result<(), ResponseCode> {
    let body = hyper::body::to_bytes(body)
        .await
        .map_err(|_| ResponseCode::BadRequest)?;
    let event: Event =
        serde_json::from_slice(&body).map_err(|_| ResponseCode::InternalServerError)?;
    println!("{}", event.msg.payload);

    authorize(event.uuid).await?;

    save_message_to_file(event.msg).map_err(|_| ResponseCode::InternalServerError)?;
    Ok(())
}

async fn handle_connection(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/pair") => {
            let token_string = generate_token();
            let write_result = save_token_to_file(&token_string);
            let response: Response<Body> = match write_result {
                Ok(_) => Response::new(Body::from(token_string)),
                Err(_e) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap(),
            };
            Ok(response)
        }

        (&Method::POST, "/messages") => {
            let body = req.into_body();
            match handle_message(body).await {
                Ok(_) => Ok(Response::default()),
                Err(err) => {
                    let status_code = match err {
                        ResponseCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
                        ResponseCode::BadRequest => StatusCode::BAD_REQUEST,
                        ResponseCode::Unauthorized => StatusCode::UNAUTHORIZED,
                    };
                    Ok(Response::builder()
                        .status(status_code)
                        .body(Body::empty())
                        .unwrap())
                }
            }
        }

        // for other routes return 404 Not Found
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 0).into();
    let service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(handle_connection)) });
    let server = Server::bind(&addr).serve(service);

    println!("Listening on http://{}", server.local_addr());

    server.await?;
    Ok(())
}
