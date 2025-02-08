use reqwest::Client;
use warp::hyper::Body;
use warp::{reply::Response, Filter, Rejection};

pub async fn start_proxy_server() {
    let client = Client::new();

    let proxy = warp::path!("proxy")
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and_then(move |params: std::collections::HashMap<String, String>| {
            let client = client.clone();
            async move {
                if let Some(url) = params.get("url") {
                    let resp = client
                        .get(url)
                        .send()
                        .await
                        .map_err(|_| warp::reject::not_found())?;

                    let status = resp.status();
                    let headers = resp.headers().clone();
                    let body = resp.bytes().await.unwrap_or_default();
                    let mut response = Response::new(Body::from(body));
                    *response.status_mut() = status;
                    *response.headers_mut() = headers;
                    Ok::<_, Rejection>(response)
                } else {
                    Err(warp::reject::not_found())
                }
            }
        });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS", "PUT", "HEAD"])
        .allow_headers(vec!["Content-Type"]);

    let routes = proxy.with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
