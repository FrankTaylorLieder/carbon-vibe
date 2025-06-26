use axum::{
    response::Html,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower::ServiceBuilder;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CarbonIntensityData {
    data: Vec<CarbonIntensityEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CarbonIntensityEntry {
    intensity: IntensityData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct IntensityData {
    actual: Option<i32>,
    forecast: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GenerationMixData {
    data: GenerationMixEntry,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GenerationMixEntry {
    #[serde(rename = "generationmix")]
    generation_mix: Vec<FuelSource>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FuelSource {
    fuel: String,
    perc: f64,
}


async fn fetch_carbon_data() -> Result<(i32, Vec<FuelSource>), Box<dyn std::error::Error>> {
    // Fetch current intensity
    let intensity_response = reqwest::get("https://api.carbonintensity.org.uk/intensity").await?;
    let intensity_data: CarbonIntensityData = intensity_response.json().await?;
    let intensity = intensity_data.data.first()
        .and_then(|entry| entry.intensity.actual.or(entry.intensity.forecast))
        .unwrap_or(0);

    // Fetch generation mix
    let mix_response = reqwest::get("https://api.carbonintensity.org.uk/generation").await?;
    let mix_data: GenerationMixData = mix_response.json().await?;
    let generation_mix = mix_data.data.generation_mix;

    Ok((intensity, generation_mix))
}

async fn serve_app() -> Html<String> {
    // Fetch data server-side
    let (intensity, generation_mix) = match fetch_carbon_data().await {
        Ok(data) => {
            println!("Successfully fetched data: intensity={}, mix_items={}", data.0, data.1.len());
            data
        },
        Err(e) => {
            println!("Error fetching data: {}", e);
            (0, vec![])
        }
    };
    
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Carbon Intensity Dashboard</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        h1 {{ text-align: center; color: #333; margin-bottom: 30px; }}
        .dashboard {{ display: grid; grid-template-columns: 1fr 1fr; gap: 30px; }}
        .intensity-display {{ background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); text-align: center; }}
        .intensity-value {{ font-size: 3em; font-weight: bold; color: #2c3e50; margin: 20px 0; }}
        .unit {{ font-size: 0.4em; color: #7f8c8d; }}
        .generation-mix {{ background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .chart-container {{ display: flex; justify-content: center; margin: 20px 0; }}
        .legend-items {{ display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }}
        .legend-item {{ display: flex; align-items: center; gap: 10px; }}
        .legend-color {{ width: 20px; height: 20px; border-radius: 3px; }}
        .legend-label {{ flex: 1; }}
        .legend-value {{ font-weight: bold; }}
        .loading {{ text-align: center; font-size: 1.5em; color: #7f8c8d; }}
        h2 {{ color: #2c3e50; margin-bottom: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>UK Carbon Intensity Dashboard</h1>
        <div class="dashboard">
            <div class="intensity-display">
                <h2>Current Carbon Intensity</h2>
                <div class="intensity-value">
                    {}
                    <span class="unit"> gCO₂/kWh</span>
                </div>
            </div>
            <div class="generation-mix">
                <h2>Energy Generation Mix</h2>
                <div class="chart-container">
                    <svg width="450" height="450" viewBox="0 0 500 500">
                        {}
                    </svg>
                </div>
                <div class="legend">
                    <div class="legend-items">
                        {}
                    </div>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#,
        intensity,
        render_pie_chart(&generation_mix),
        render_legend(&generation_mix)
    );
    
    Html(html)
}

fn render_pie_chart(generation_mix: &[FuelSource]) -> String {
    let colors = vec![
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FECA57",
        "#FF9FF3", "#54A0FF", "#5F27CD", "#00D2D3", "#FF9F43",
        "#EE5A24", "#0ABDE3", "#10AC84", "#F79F1F", "#A3CB38"
    ];
    
    let total: f64 = generation_mix.iter().map(|f| f.perc).sum();
    let mut start_angle = 0.0;
    let mut elements = String::new();
    
    for (i, fuel) in generation_mix.iter().enumerate() {
        let percentage = fuel.perc / total;
        let angle = percentage * 2.0 * std::f64::consts::PI;
        let end_angle = start_angle + angle;
        
        // Skip very small segments for labels but still draw them
        let show_label = fuel.perc >= 0.5;
        
        let center_x = 250.0;
        let center_y = 250.0;
        let radius = 150.0;
        
        let x1 = center_x + radius * start_angle.cos();
        let y1 = center_y + radius * start_angle.sin();
        let x2 = center_x + radius * end_angle.cos();
        let y2 = center_y + radius * end_angle.sin();
        
        let large_arc = if angle > std::f64::consts::PI { 1 } else { 0 };
        
        // Create pie segment path
        let path = format!(
            "M {} {} L {} {} A {} {} 0 {} 1 {} {} Z",
            center_x, center_y, x1, y1, radius, radius, large_arc, x2, y2
        );
        
        let color = colors.get(i % colors.len()).unwrap_or(&"#999999");
        
        // Add pie segment
        elements.push_str(&format!(
            r#"<path d="{}" fill="{}" stroke="white" stroke-width="2" />"#,
            path, color
        ));
        
        // Add label only for segments that are large enough
        if show_label {
            // Calculate label position (middle of arc, closer to the pie)
            let mid_angle = start_angle + angle / 2.0;
            let label_radius = 175.0; // Closer to the pie edge
            
            let label_x = center_x + label_radius * mid_angle.cos();
            let label_y = center_y + label_radius * mid_angle.sin();
            
            // Center-align all text
            let text_anchor = "middle";
            
            // Add label text (closer to pie, no connecting line)
            elements.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\" font-family=\"Arial, sans-serif\" font-size=\"11\" font-weight=\"bold\" fill=\"#333333\">{}</text>",
                label_x, label_y - 2.0, text_anchor, fuel.fuel
            ));
            
            // Add percentage on a second line
            elements.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\" font-family=\"Arial, sans-serif\" font-size=\"10\" fill=\"#666666\">{:.1}%</text>",
                label_x, label_y + 10.0, text_anchor, fuel.perc
            ));
        }
        
        start_angle = end_angle;
    }
    
    elements
}

fn render_legend(generation_mix: &[FuelSource]) -> String {
    let colors = vec![
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FECA57",
        "#FF9FF3", "#54A0FF", "#5F27CD", "#00D2D3", "#FF9F43",
        "#EE5A24", "#0ABDE3", "#10AC84", "#F79F1F", "#A3CB38"
    ];
    
    generation_mix.iter().enumerate().map(|(i, fuel)| {
        let color = colors.get(i % colors.len()).unwrap_or(&"#999999");
        format!(
            r#"<div class="legend-item">
                <div class="legend-color" style="background-color: {}"></div>
                <span class="legend-label">{}</span>
                <span class="legend-value">{:.1}%</span>
            </div>"#,
            color, fuel.fuel, fuel.perc
        )
    }).collect::<Vec<_>>().join("")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(serve_app))
        .layer(ServiceBuilder::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}