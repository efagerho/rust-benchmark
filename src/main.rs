use clap::Parser;
use futures::future;
use hyper::{Body, Client, Method, Request, StatusCode};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
static ERROR_COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long, default_value = "http://127.0.0.1:8080")]
    address: String,

    #[arg(short, long, default_value_t = 1000)]
    connections: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = Client::new();

    future::join_all((0..(args.connections + 1)).map(|i| {
        let client = client.clone();
        let address = args.address.clone();
        if i == 0 {
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    println!(
                        "Made {} requests and got {} errors",
                        REQUEST_COUNT.load(Ordering::SeqCst),
                        ERROR_COUNT.load(Ordering::SeqCst)
                    );
                }
            })
        } else {
            tokio::spawn(async move {
                loop {
                    let request = Request::builder()
                        .method(Method::GET)
                        .uri(address.clone())
                        .body(Body::from(""))
                        .unwrap();
                    let res = client.request(request).await;
                    match res {
                        Ok(res) => {
                            if res.status() == StatusCode::OK {
                                REQUEST_COUNT.fetch_add(1, Ordering::SeqCst);
                                continue;
                            }
                        }
                        Err(e) => println!("Got error: {:?}", e),
                    }
                    ERROR_COUNT.fetch_add(1, Ordering::SeqCst);
                }
            })
        }
    }))
    .await;
}
