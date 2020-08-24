use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::fs::OpenOptions;
use std::io::{BufReader, Seek, SeekFrom, Write};

type Token = String;

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

        // for other routes return 404 Not Found
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();
    let service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(handle_connection)) });
    let server = Server::bind(&addr).serve(service);

    println!("Listening on http://{}", server.local_addr());

    server.await?;
    Ok(())
}
