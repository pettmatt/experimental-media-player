use hyper::{header, Request, Uri};
use hyper::{Client, Uri};
use serde_json::Value;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let apis: Vec<Api> = fetch_apis("requests.json");
    let mut client = hyper::Client::new();
    let mut responses = Vec::new();

    for api in apis {}

    println!("Hello responses: {:?}", responses);
    Ok(())
}

async fn fetch_url(client: Client, url: Uri) -> Result {
    let res = client.get(url).await?;
    let body = hyper::body::to_bytes(res.into_body()).await?;
    Ok(body)
}

fn fetch_apis(path: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    use std::fs;
    let contents = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&contents)?;
    let list = Vec::new();

    for object in json {
        let mut api = Api {
            url: object.url,
            headers: std::collections::HashMap::new(),
        };

        for h in object.headers {
            api.headers.set(h.0, h.1);
        }

        list.push(api)
    }

    Ok(list)
}

enum RequestMethod {
    GET,
    POST,
}

struct Api<'a> {
    url: &'a str,
    headers: std::collections::HashMap<&'a str, &'a str>,
    // body: hyper::ffi::hyper_body,
}

impl Api<'_> {
    async fn default_request(&self, method: RequestMethod) -> Request {
        let uri = &self.url.parse::<Uri>()?;
        let authority = uri.authority().unwrap().clone();

        let mut request = Request::builder()
            .header(header::HOST, authority.as_str())
            .header("content-type", "application/json")
            .uri(uri)
            .body(Empty::<Bytes>::new())?;

        match method {
            RequestMethod::GET => request.method(hyper::Method::GET),
            RequestMethod::POST => request.method(hyper::Method::POST),
            _ => panic!("Unsupported method argument '{:?}' passed", method),
        }

        request
    }

    fn add_headers(&mut self, request: Request<T>) -> Request<T> {
        for h in &self.headers {
            request.header(h.0, h.1)?;
        }

        request
    }

    fn make_request(&self) {
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header("content-type", "application/json")
            .uri(self.url)
            .body(Body::empty())?;

        for h in self.headers {
            request.header(h.0, h.1)?;
        }

        let response = client.request(request).await?;
        let body = response.into_body();
        let mut data = null;

        while let Some(chunk) = body.data().await {
            let bytes = chunk.expect("Body chunk");
            responses.push(bytes);
        }
    }
}
