use serde::Deserialize;

#[derive(Deserialize)]
struct CarbonIntensityData {
    data: Vec<CarbonIntensityEntry>,
}

#[derive(Deserialize)]
struct CarbonIntensityEntry {
    intensity: IntensityData,
}

#[derive(Deserialize)]
struct IntensityData {
    actual: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.carbonintensity.org.uk/intensity";
    
    let response = reqwest::get(url).await?;
    let carbon_data: CarbonIntensityData = response.json().await?;
    
    if let Some(entry) = carbon_data.data.first() {
        println!("{}", entry.intensity.actual);
    }
    
    Ok(())
}
