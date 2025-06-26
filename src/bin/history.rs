use serde::Deserialize;
use tracing::{trace, instrument};

#[derive(Deserialize, Debug)]
struct CarbonIntensityData {
    data: Vec<CarbonIntensityEntry>,
}

#[derive(Deserialize, Debug)]
struct CarbonIntensityEntry {
    from: String,
    #[allow(dead_code)]
    to: String,
    intensity: IntensityData,
}

#[derive(Deserialize, Debug)]
struct IntensityData {
    actual: Option<i32>,
    forecast: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = match std::env::var("RUST_LOG") {
        Ok(level) if level == "trace" => "history=trace,warn".to_string(),
        Ok(level) => level,
        Err(_) => "info".to_string(),
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .init();

    fetch_carbon_intensity_history().await
}

#[instrument]
async fn fetch_carbon_intensity_history() -> Result<(), Box<dyn std::error::Error>> {
    // Calculate the time range for the last 12 hours
    let now = chrono::Utc::now();
    let twelve_hours_ago = now - chrono::Duration::hours(12);
    
    let from_date = twelve_hours_ago.format("%Y-%m-%dT%H:%MZ").to_string();
    let to_date = now.format("%Y-%m-%dT%H:%MZ").to_string();
    
    let url = format!(
        "https://api.carbonintensity.org.uk/intensity/{from_date}/{to_date}",
        from_date = from_date,
        to_date = to_date
    );
    
    trace!("Making API request to: {}", url);
    let response = reqwest::get(&url).await?;
    
    trace!("Received response with status: {}", response.status());
    let response_text = response.text().await?;
    trace!("Raw response body: {}", response_text);
    
    let carbon_data: CarbonIntensityData = serde_json::from_str(&response_text)?;
    trace!("Parsed response data: {:?}", carbon_data);
    
    // Group by hour and calculate average intensity
    let mut hourly_data: std::collections::BTreeMap<String, Vec<i32>> = std::collections::BTreeMap::new();
    
    for entry in carbon_data.data {
        let datetime = chrono::DateTime::parse_from_str(&entry.from, "%Y-%m-%dT%H:%M%#z")
            .map_err(|e| format!("Failed to parse datetime: {}", e))?;
        
        let hour_key = datetime.format("%Y-%m-%d %H:00").to_string();
        let intensity = entry.intensity.actual
            .or(entry.intensity.forecast)
            .unwrap_or(0);
        
        hourly_data.entry(hour_key).or_default().push(intensity);
    }
    
    // Print hourly averages
    for (hour, intensities) in hourly_data {
        let avg_intensity = intensities.iter().sum::<i32>() / intensities.len() as i32;
        println!("{hour}: {intensity}", hour = hour, intensity = avg_intensity);
    }
    
    Ok(())
}