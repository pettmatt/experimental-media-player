use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, path::Path};
use tokio;

type CResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> CResult<()> {
    let apis = fetch_apis("./src/requests.json").unwrap();

    for api in &apis {}

    println!("Hello apis: {:?}", &apis);
    Ok(())
}

fn fetch_apis(path_str: &str) -> CResult<Vec<Api>> {
    let path = Path::new(path_str);
    let contents = std::fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&contents)?;
    let list: Vec<Api> = Vec::new();

    println!("JSON:");
    for object in json.as_array().unwrap() {
        println!("{:?}", object);
        let mut api = Api {
            url: String::from(object.url),
            paths: HashMap::new(),
        };

        for p in object.paths {
            let mut path = ApiPath {
                path: String::from(p.path),
                headers: http::header::HeaderMap::new(),
                body_parameters: HashMap::new(),
            };

            for h in p.headers {
                api.headers.insert(h.0, h.1);
            }

            for bp in p.body {
                api.body_parameters.insert(bp.0, bp.1);
            }

            list.push(api)
        }
    }

    Ok(list)
}

#[derive(Debug, Deserialize)]
struct ApiPath {
    path: String,
    #[serde(deserialize_with = "deserialize_header_map")]
    headers: http::header::HeaderMap,
    body_parameters: HashMap<String, String>,
}

impl<'a> std::fmt::Display for ApiPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Path {{ path: {}, headers: {:?}, body_parameters: {:?} }}",
            self.path, self.headers, self.body_parameters
        )
    }
}

#[derive(Debug, Deserialize)]
struct Api {
    url: String,
    paths: HashMap<String, ApiPath>,
}

impl Api {
    async fn default_request(&self, subpath: &ApiPath) -> CResult<reqwest::Response> {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}{}", &self.url, subpath))
            .form(&subpath.body_parameters)
            .header("content-type", "application/json")
            .headers(subpath.headers.clone())
            .send()
            .await?;

        println!("Request response {:?}", response);
        Ok(response)
    }
}

fn deserialize_header_map<'de, D>(deserializer: D) -> Result<http::header::HeaderMap, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map: HashMap<String, String> = HashMap::deserialize(deserializer)?;

    let mut header_map = http::header::HeaderMap::new();
    for (key, value) in map {
        if let (Ok(header_name), Ok(header_value)) = (
            http::header::HeaderName::from_bytes(key.as_bytes()),
            http::header::HeaderValue::from_str(&value),
        ) {
            header_map.insert(header_name, header_value);
        }
    }
    Ok(header_map)
}
