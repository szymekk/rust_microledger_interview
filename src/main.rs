use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

async fn handle_connection(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/pair") => {
            let response = Response::new(Body::from("hello"));
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
