use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, path::Path, str::FromStr};
use tokio;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let apis = fetch_apis("./src/requests.json").unwrap();

    for api in &apis {}

    println!("Hello apis: {:?}", &apis);
    Ok(())
}

fn fetch_apis(path_str: &str) -> Result<Vec<Api>> {
    let path = Path::new(path_str);
    let contents = std::fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&contents)?;
    let mut list: Vec<Api> = Vec::new();

    for object in json.as_array().unwrap() {
        let mut api = Api {
            url: object["url"].to_string(),
            paths: HashMap::new(),
        };

        for (name, p) in object["paths"].as_object().unwrap() {
            let mut path = ApiPath {
                path: p["path"].to_string(),
                headers: http::header::HeaderMap::new(),
                body_parameters: HashMap::new(),
            };

            for header in p["headers"].as_array().unwrap() {
                for (key, value) in header.as_object().unwrap() {
                    path.headers.insert(
                        http::header::HeaderName::try_from(key.to_string()).unwrap(),
                        http::header::HeaderValue::try_from(value.to_string()).unwrap(),
                    );
                }
            }

            for bp in p["body"].as_array().unwrap() {
                for (key, value) in bp.as_object().unwrap() {
                    path.body_parameters.insert(key.clone(), value.to_string());
                }
            }

            api.paths.insert(name.clone(), path);
            list.push(api.clone());
        }
    }

    Ok(list)
}

#[derive(Debug, Clone)]
struct ApiPath {
    path: String,
    // #[serde(deserialize_with = "deserialize_header_map")]
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

#[derive(Debug, Clone)]
struct Api {
    url: String,
    paths: HashMap<String, ApiPath>,
}

impl Api {
    async fn default_request(&self, subpath: &ApiPath) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let url = format!("{}{}", &self.url, subpath);
        let response = client
            .post(url)
            .form(&subpath.body_parameters)
            .header("content-type", "application/json")
            .headers(subpath.headers.clone())
            .send()
            .await?;

        println!("Request response {:?}", response);
        Ok(response)
    }
}

fn deserialize_header_map<'de, D>(
    deserializer: D,
) -> std::result::Result<http::header::HeaderMap, D::Error>
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
