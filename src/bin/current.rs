use serde::Deserialize;
use tracing::{trace, instrument};

#[derive(Deserialize, Debug)]
struct CarbonIntensityData {
    data: Vec<CarbonIntensityEntry>,
}

#[derive(Deserialize, Debug)]
struct CarbonIntensityEntry {
    intensity: IntensityData,
}

#[derive(Deserialize, Debug)]
struct IntensityData {
    actual: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = match std::env::var("RUST_LOG") {
        Ok(level) if level == "trace" => "current=trace,warn".to_string(),
        Ok(level) => level,
        Err(_) => "info".to_string(),
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .init();

    fetch_carbon_intensity().await
}

#[instrument]
async fn fetch_carbon_intensity() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.carbonintensity.org.uk/intensity";
    
    trace!("Making API request to: {}", url);
    let response = reqwest::get(url).await?;
    
    trace!("Received response with status: {}", response.status());
    let response_text = response.text().await?;
    trace!("Raw response body: {}", response_text);
    
    let carbon_data: CarbonIntensityData = serde_json::from_str(&response_text)?;
    trace!("Parsed response data: {:?}", carbon_data);
    
    if let Some(entry) = carbon_data.data.first() {
        println!("{intensity}", intensity = entry.intensity.actual);
    }
    
    Ok(())
}