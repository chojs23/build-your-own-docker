use flate2::read::GzDecoder;
use reqwest::Client;
use serde::Deserialize;
use std::io::{Cursor, Seek, SeekFrom};
use tar::Archive;
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageLayer {
    pub media_type: String,
    pub digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageManifest {
    pub layers: Vec<ImageLayer>,
}

pub async fn download_image(image: &str, dir: &TempDir) {
    let (image_name, image_tag) = parse_image(image);
    let client = Client::new();
    let token = auth(&client, &image_name).await;
    let image_manifest = fetch_image_manifest(&client, &image_name, &image_tag, &token).await;
    download_image_from_manifest(&client, &image_name, &token, &image_manifest, dir).await;
}

async fn fetch_image_manifest(
    client: &Client,
    image_name: &String,
    image_tag: &String,
    token: &String,
) -> ImageManifest {
    let request =
        format!("https://registry.hub.docker.com/v2/library/{image_name}/manifests/{image_tag}");
    client
        .get(request)
        .bearer_auth(token)
        .header(
            "Accept",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .send()
        .await
        .expect("failed to fetch manifest")
        .json()
        .await
        .expect("failed to deserialize manifest")
}

async fn download_image_from_manifest(
    client: &Client,
    image_name: &String,
    token: &String,
    manifest: &ImageManifest,
    dir: &TempDir,
) {
    for layer in manifest.layers.iter() {
        let request = format!(
            "https://registry.hub.docker.com/v2/library/{image_name}/blobs/{}",
            &layer.digest
        );
        let image_layer_response = client
            .get(request)
            .bearer_auth(token)
            .header(reqwest::header::ACCEPT, &layer.media_type)
            .send()
            .await
            .expect("failed to download image layer")
            .bytes()
            .await
            .expect("failed to get back bytes for layer");
        let mut bytes = Cursor::new(image_layer_response);
        let mut file = tempfile::tempfile().expect("failed to create tempfile");
        std::io::copy(&mut bytes, &mut file).expect("failed to copy layer bytes to temp file");
        file.seek(SeekFrom::Start(0))
            .expect("failed to start seeking at beginning of file");
        let decoded = GzDecoder::new(file);
        Archive::new(decoded)
            .unpack(dir)
            .expect("failed to unpack archive");
    }
}

fn parse_image(image: &str) -> (String, String) {
    let parsed_image_str: Vec<&str> = image.split(':').collect();
    if parsed_image_str.len() == 1 {
        return (parsed_image_str[0].to_string(), "latest".to_string());
    }
    if parsed_image_str.len() == 2 {
        let (name, tag) = (parsed_image_str[0], parsed_image_str[1]);
        return (name.to_string(), tag.to_string());
    }
    panic!("Invalid image name: {}", image);
}

async fn auth(client: &Client, image_name: &String) -> String {
    let request = format!(
        "https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/{image_name}:pull",
    );
    let response: TokenResponse = client
        .get(request)
        .send()
        .await
        .expect("failed to send request")
        .json()
        .await
        .expect("failed to deserialize json response");
    response.token
}
