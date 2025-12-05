use base64::{Engine as _, engine::general_purpose};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;
use std::{collections::HashMap, path::Path};
use tokio;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
struct AuthResponse {
    url: reqwest::Url,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::from_filename(".env-requests").ok(); // Load env files
    let apis = fetch_apis("./src/apis.json").unwrap();
    println!("APIs {:?}", apis);

    if let Some(api) = apis.get("spotify") {
        println!("\nExecuting auth for {}", api.url);
        let response = api.authenticate().await;

        if let Ok((result, state)) = response {
            if let Ok(response) = result.error_for_status() {
                let json: AuthResponse = response.json().await?;
                let state_param = json
                    .url
                    .query_pairs()
                    .find(|(key, _)| key == "sort")
                    .map(|(_, value)| value.to_string());

                if let Some(state_value) = state_param {
                    if state_value == state {
                        println!("Response can be trusted!");
                        println!("\nResult {:?}", json);
                    } else {
                        println!("Response CANNOT be trusted!");
                    }
                };
            } else {
                println!("Authentication failed. Status contained an error code.");
            }
        }
    }

    Ok(())
}

fn fetch_apis(path_str: &str) -> Result<HashMap<String, Api>> {
    let path = Path::new(path_str);
    let contents = std::fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&contents)?;
    let mut list: HashMap<String, Api> = HashMap::new();

    for object in json.as_array().unwrap() {
        let mut api = Api {
            url: String::from(object["url"].as_str().unwrap()),
            paths: HashMap::new(),
            ..Default::default()
        };

        for (name, p) in object["paths"].as_object().unwrap() {
            let mut path = ApiPath {
                path: String::from(p["path"].as_str().unwrap()),
                headers: http::header::HeaderMap::new(),
                body_parameters: HashMap::new(),
                path_parameters: HashMap::new(),
            };

            for (key, value) in p["headers"].as_object().unwrap() {
                let v = String::from(value.as_str().unwrap());
                path.headers.insert(
                    http::header::HeaderName::try_from(key).unwrap(),
                    http::header::HeaderValue::try_from(v).unwrap(),
                );
            }

            for (key, value) in p["body"].as_object().unwrap() {
                let v = String::from(value.as_str().unwrap());
                path.body_parameters.insert(key.clone(), v);
            }

            for (key, value) in p["path-parameters"].as_object().unwrap() {
                let v = String::from(value.as_str().unwrap());
                path.path_parameters.insert(key.clone(), v);
            }

            api.paths.insert(name.clone(), path);
        }

        let s_url: Vec<&str> = api.url.split(".").collect();
        let key = s_url.get(1).unwrap_or(&"NO-DOMAIN");
        list.insert(key.to_string(), api.clone());
    }

    Ok(list)
}

#[derive(Debug, Clone, Default)]
struct ApiPath {
    path: String,
    // #[serde(deserialize_with = "deserialize_header_map")]
    headers: http::header::HeaderMap,
    body_parameters: HashMap<String, String>,
    path_parameters: HashMap<String, String>,
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

impl ApiPath {
    fn stringify_path_parametrs(&self) -> String {
        self.path_parameters
            .iter()
            .map(|(k, v)| format!("{}={}{}", k, v, "&"))
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
struct Authentication {
    scope: Vec<String>,
    valid: bool,
    expires: i32,
    hash: String,
}

// Accepted characters to generate random strings with
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
abcdefghijklmnopqrstuvwxyz\
0123456789";

impl Authentication {
    /// Generates a random alphanumeric string of the given length.
    ///
    /// # Arguments
    /// * `length` - The length of the random string (as `i32`).
    ///
    /// # Returns
    /// A random alphanumeric string of the specified length.
    ///
    /// # Panics
    /// Panics if `length` is negative.
    fn generate_random_string(&self, length: i32) -> String {
        use rand::Rng;
        assert!(length >= 0, "Length must be non-negative");

        let mut rng = rand::rng();
        (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn sha256(&self, raw: String) -> String {
        let mut encoder = Sha256::new();
        let data = raw.as_bytes();
        encoder.update(data);
        let result = encoder.finalize();
        format!("{:x}", result)
    }

    fn base64(&self, hash: String) -> String {
        general_purpose::STANDARD.encode(hash.as_bytes())
    }
}

#[derive(Debug, Clone, Default)]
struct Api {
    url: String,
    paths: HashMap<String, ApiPath>,
    authentication: Authentication,
}

impl Api {
    async fn authenticate(&self) -> Result<(reqwest::Response, String)> {
        let state = self.authentication.generate_random_string(16);
        let raw_state = self.authentication.generate_random_string(64);
        let hash = self.authentication.sha256(raw_state);
        let challenge = self.authentication.base64(hash);

        let mut auth_path = self.paths.get("authenticate").cloned().unwrap();
        auth_path
            .path_parameters
            .insert("state".to_string(), state.clone());
        auth_path
            .path_parameters
            .insert("code_challenge".to_string(), challenge);

        let redirect = env::var("SPOTIFY_REDIRECT_URI").unwrap_or_else(|_| "NONE".to_string());
        let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap_or_else(|_| "NONE".to_string());

        auth_path
            .path_parameters
            .entry("redirect_uri".to_string())
            .and_modify(|v| *v = redirect.clone())
            .or_insert(redirect);
        auth_path
            .path_parameters
            .entry("client_id".to_string())
            .and_modify(|v| *v = client_id.clone())
            .or_insert(client_id);

        let response = self.get_request(&auth_path).await.unwrap();
        Ok((response, state))
    }

    async fn get_request(&self, subpath: &ApiPath) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}{}{}",
            &self.url,
            subpath.path,
            subpath.stringify_path_parametrs()
        );

        let response = client
            .get(url)
            .header("content-type", "application/json")
            .headers(subpath.headers.clone())
            .send()
            .await?;

        Ok(response)
    }

    async fn post_request(&self, subpath: &ApiPath) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let url = format!("{}{}", &self.url, subpath.path);
        let response = client
            .post(url)
            .form(&subpath.body_parameters)
            .header("content-type", "application/json")
            .headers(subpath.headers.clone())
            .send()
            .await?;

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
