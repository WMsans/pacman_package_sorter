use crate::error::AppError;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct AurPackage {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Popularity")]
    pub popularity: f64,
    #[serde(rename = "NumVotes")]
    pub num_votes: u32,
}

#[derive(Debug, Deserialize)]
struct AurResponse {
    results: Vec<AurPackage>,
}

// Fetch package data from AUR
pub async fn fetch_aur_package_data(
    package_names: Vec<String>,
) -> Result<HashMap<String, AurPackage>, AppError> {
    if package_names.is_empty() {
        return Ok(HashMap::new());
    }

    let client = reqwest::Client::new();
    let response: AurResponse = client
        .get("https://aur.archlinux.org/rpc/v5/info")
        .query(
            &package_names
                .into_iter()
                .map(|name| ("arg[]", name))
                .collect::<Vec<_>>(),
        )
        .send()
        .await?
        .json()
        .await?;

    let mut aur_data = HashMap::new();
    for pkg in response.results {
        aur_data.insert(pkg.name.clone(), pkg);
    }

    Ok(aur_data)
}