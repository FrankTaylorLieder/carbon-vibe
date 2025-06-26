use axum::{Router, response::Html, routing::get};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower::ServiceBuilder;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CarbonIntensityData {
    data: Vec<CarbonIntensityEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CarbonIntensityEntry {
    from: Option<String>,
    #[allow(dead_code)]
    to: Option<String>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CarbonFactorsData {
    data: Vec<CarbonFactors>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CarbonFactors {
    #[serde(rename = "Biomass")]
    biomass: i32,
    #[serde(rename = "Coal")]
    coal: i32,
    #[serde(rename = "Gas (Combined Cycle)")]
    gas_combined_cycle: i32,
    #[serde(rename = "Gas (Open Cycle)")]
    gas_open_cycle: i32,
    #[serde(rename = "Hydro")]
    hydro: i32,
    #[serde(rename = "Nuclear")]
    nuclear: i32,
    #[serde(rename = "Other")]
    other: i32,
    #[serde(rename = "Solar")]
    solar: i32,
    #[serde(rename = "Wind")]
    wind: i32,
    #[serde(rename = "Dutch Imports")]
    dutch_imports: i32,
    #[serde(rename = "French Imports")]
    french_imports: i32,
    #[serde(rename = "Irish Imports")]
    irish_imports: i32,
}

#[derive(Clone, Debug)]
struct FuelSourceWithIntensity {
    fuel: String,
    perc: f64,
    carbon_intensity: i32,
}

#[derive(Clone, Debug)]
struct IntensityPoint {
    datetime: String,
    intensity: i32,
    is_forecast: bool,
}

async fn fetch_carbon_data()
-> Result<(i32, Vec<FuelSourceWithIntensity>, Vec<IntensityPoint>), Box<dyn std::error::Error>> {
    // Fetch current intensity
    let intensity_response = reqwest::get("https://api.carbonintensity.org.uk/intensity").await?;
    let intensity_data: CarbonIntensityData = intensity_response.json().await?;
    let intensity = intensity_data
        .data
        .first()
        .and_then(|entry| entry.intensity.actual.or(entry.intensity.forecast))
        .unwrap_or(0);

    // Fetch generation mix
    let mix_response = reqwest::get("https://api.carbonintensity.org.uk/generation").await?;
    let mix_data: GenerationMixData = mix_response.json().await?;
    let generation_mix = mix_data.data.generation_mix;

    // Fetch carbon factors
    let factors_response =
        reqwest::get("https://api.carbonintensity.org.uk/intensity/factors").await?;
    let factors_data: CarbonFactorsData = factors_response.json().await?;
    let factors = factors_data
        .data
        .first()
        .ok_or("No factors data available")?;

    // Combine generation mix with carbon intensity factors
    let enriched_mix = generation_mix
        .into_iter()
        .map(|fuel| {
            let carbon_intensity = match fuel.fuel.as_str() {
                "biomass" => factors.biomass,
                "coal" => factors.coal,
                "gas" => factors.gas_combined_cycle, // Default to combined cycle
                "hydro" => factors.hydro,
                "nuclear" => factors.nuclear,
                "other" => factors.other,
                "solar" => factors.solar,
                "wind" => factors.wind,
                "imports" => {
                    (factors.dutch_imports + factors.french_imports + factors.irish_imports) / 3
                } // Average imports
                _ => 0,
            };

            FuelSourceWithIntensity {
                fuel: fuel.fuel,
                perc: fuel.perc,
                carbon_intensity,
            }
        })
        .collect();

    // Fetch 24-hour timeline data (12 hours past + 12 hours future)
    let now = chrono::Utc::now();
    let twelve_hours_ago = now - chrono::Duration::hours(12);
    let twelve_hours_future = now + chrono::Duration::hours(12);

    let from_date = twelve_hours_ago.format("%Y-%m-%dT%H:%MZ").to_string();
    let to_date = twelve_hours_future.format("%Y-%m-%dT%H:%MZ").to_string();

    let timeline_url = format!(
        "https://api.carbonintensity.org.uk/intensity/{from_date}/{to_date}",
        from_date = from_date,
        to_date = to_date
    );

    let timeline_response = reqwest::get(&timeline_url).await?;
    let timeline_data: CarbonIntensityData = timeline_response.json().await?;

    // Process timeline data into points
    let timeline_points: Vec<IntensityPoint> = timeline_data
        .data
        .into_iter()
        .filter_map(|entry| {
            let datetime = entry.from?;
            let intensity = entry
                .intensity
                .actual
                .unwrap_or(entry.intensity.forecast.unwrap_or(0));
            let is_forecast = entry.intensity.actual.is_none();

            Some(IntensityPoint {
                datetime,
                intensity,
                is_forecast,
            })
        })
        .collect();

    Ok((intensity, enriched_mix, timeline_points))
}

async fn serve_app() -> Html<String> {
    // Fetch data server-side
    let (intensity, generation_mix, timeline_points) = match fetch_carbon_data().await {
        Ok(data) => {
            println!(
                "Successfully fetched data: intensity={}, mix_items={}, timeline_points={}",
                data.0,
                data.1.len(),
                data.2.len()
            );
            data
        }
        Err(e) => {
            println!("Error fetching data: {error}", error = e);
            (0, vec![], vec![])
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
        .legend-items {{ display: grid; grid-template-columns: 1fr 1fr; gap: 15px; }}
        .legend-item {{ display: flex; align-items: center; gap: 12px; }}
        .legend-color {{ width: 20px; height: 20px; border-radius: 3px; flex-shrink: 0; }}
        .legend-info {{ display: flex; flex-direction: column; }}
        .legend-label {{ font-weight: bold; color: #2c3e50; }}
        .legend-details {{ font-size: 0.9em; color: #7f8c8d; margin-top: 2px; }}
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
                    {intensity}
                    <span class="unit"> gCO₂/kWh</span>
                </div>
                <div class="chart-container">
                    {intensity_chart}
                </div>
            </div>
            <div class="generation-mix">
                <h2>Energy Generation Mix</h2>
                <div class="chart-container">
                    <svg width="450" height="450" viewBox="0 0 500 500">
                        {pie_chart}
                    </svg>
                </div>
                <div class="legend">
                    <div class="legend-items">
                        {legend}
                    </div>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#,
        intensity = intensity,
        intensity_chart = render_intensity_chart(&timeline_points),
        pie_chart = render_pie_chart(&generation_mix),
        legend = render_legend(&generation_mix)
    );

    Html(html)
}

fn render_pie_chart(generation_mix: &[FuelSourceWithIntensity]) -> String {
    let colors = vec![
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FECA57", "#FF9FF3", "#54A0FF", "#5F27CD",
        "#00D2D3", "#FF9F43", "#EE5A24", "#0ABDE3", "#10AC84", "#F79F1F", "#A3CB38",
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
            "M {center_x} {center_y} L {x1} {y1} A {radius} {radius} 0 {large_arc} 1 {x2} {y2} Z",
            center_x = center_x,
            center_y = center_y,
            x1 = x1,
            y1 = y1,
            radius = radius,
            large_arc = large_arc,
            x2 = x2,
            y2 = y2
        );

        let color = colors.get(i % colors.len()).unwrap_or(&"#999999");

        // Add pie segment
        elements.push_str(&format!(
            r#"<path d="{path}" fill="{color}" stroke="white" stroke-width="2" />"#,
            path = path,
            color = color
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
                "<text x=\"{label_x}\" y=\"{label_y}\" text-anchor=\"{text_anchor}\" font-family=\"Arial, sans-serif\" font-size=\"11\" font-weight=\"bold\" fill=\"#333333\">{fuel_name}</text>",
                label_x = label_x,
                label_y = label_y - 2.0,
                text_anchor = text_anchor,
                fuel_name = fuel.fuel
            ));

            // Add percentage on a second line
            elements.push_str(&format!(
                "<text x=\"{label_x}\" y=\"{label_y}\" text-anchor=\"{text_anchor}\" font-family=\"Arial, sans-serif\" font-size=\"10\" fill=\"#666666\">{percentage:.1}%</text>",
                label_x = label_x,
                label_y = label_y + 10.0,
                text_anchor = text_anchor,
                percentage = fuel.perc
            ));
        }

        start_angle = end_angle;
    }

    elements
}

fn render_legend(generation_mix: &[FuelSourceWithIntensity]) -> String {
    let colors = vec![
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FECA57", "#FF9FF3", "#54A0FF", "#5F27CD",
        "#00D2D3", "#FF9F43", "#EE5A24", "#0ABDE3", "#10AC84", "#F79F1F", "#A3CB38",
    ];

    generation_mix
        .iter()
        .enumerate()
        .map(|(i, fuel)| {
            let color = colors.get(i % colors.len()).unwrap_or(&"#999999");
            let intensity_text = if fuel.carbon_intensity == 0 {
                "0 gCO₂/kWh".to_string()
            } else {
                format!("{carbon_intensity} gCO₂/kWh", carbon_intensity = fuel.carbon_intensity)
            };

            format!(
                r#"<div class="legend-item">
                <div class="legend-color" style="background-color: {color}"></div>
                <div class="legend-info">
                    <span class="legend-label">{fuel_name}</span>
                    <span class="legend-details">{percentage:.1}% • {intensity_text}</span>
                </div>
            </div>"#,
                color = color,
                fuel_name = fuel.fuel,
                percentage = fuel.perc,
                intensity_text = intensity_text
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_intensity_chart(timeline_points: &[IntensityPoint]) -> String {
    if timeline_points.is_empty() {
        return String::new();
    }

    let width = 500.0;
    let height = 180.0;
    let margin_left = 50.0;
    let margin_right = 20.0;
    let margin_top = 20.0;
    let margin_bottom = 40.0;
    let chart_width = width - margin_left - margin_right;
    let chart_height = height - margin_top - margin_bottom;

    // Find min and max intensity for scaling
    let intensities: Vec<i32> = timeline_points.iter().map(|p| p.intensity).collect();
    let min_intensity = *intensities.iter().min().unwrap_or(&0) as f64;
    let max_intensity = *intensities.iter().max().unwrap_or(&100) as f64;
    let intensity_range = max_intensity - min_intensity;

    if intensity_range == 0.0 {
        return String::new();
    }

    // Generate path data
    let mut path_data = String::new();
    let mut forecast_path_data = String::new();

    for (i, point) in timeline_points.iter().enumerate() {
        let x = margin_left + (i as f64 / (timeline_points.len() - 1) as f64) * chart_width;
        let y = margin_top + chart_height
            - ((point.intensity as f64 - min_intensity) / intensity_range) * chart_height;

        if i == 0 {
            if point.is_forecast {
                forecast_path_data = format!("M {x} {y}", x = x, y = y);
            } else {
                path_data = format!("M {x} {y}", x = x, y = y);
            }
        } else if point.is_forecast {
            if forecast_path_data.is_empty() {
                // Start forecast path from last historical point
                if let Some(prev_point) = timeline_points.get(i - 1) {
                    let prev_x = margin_left
                        + ((i - 1) as f64 / (timeline_points.len() - 1) as f64) * chart_width;
                    let prev_y = margin_top + chart_height
                        - ((prev_point.intensity as f64 - min_intensity) / intensity_range)
                            * chart_height;
                    forecast_path_data = format!("M {prev_x} {prev_y} L {x} {y}", prev_x = prev_x, prev_y = prev_y, x = x, y = y);
                } else {
                    forecast_path_data = format!("M {x} {y}", x = x, y = y);
                }
            } else {
                forecast_path_data.push_str(&format!(" L {x} {y}", x = x, y = y));
            }
        } else {
            path_data.push_str(&format!(" L {x} {y}", x = x, y = y));
        }
    }

    // Find current time marker
    let now = chrono::Utc::now();
    let current_index = timeline_points
        .iter()
        .position(|p| {
            if let Ok(point_time) = chrono::DateTime::parse_from_str(&p.datetime, "%Y-%m-%dT%H:%MZ")
            {
                point_time.timestamp() > now.timestamp()
            } else {
                false
            }
        })
        .unwrap_or(timeline_points.len() / 2);

    let current_x =
        margin_left + (current_index as f64 / (timeline_points.len() - 1) as f64) * chart_width;

    // Calculate Y-axis labels (every 20 units, rounded)
    let y_step = ((max_intensity - min_intensity) / 4.0).ceil().max(20.0);
    let y_start = (min_intensity / y_step).floor() * y_step;
    let y_end = (max_intensity / y_step).ceil() * y_step;

    // Generate Y-axis labels
    let mut y_labels = String::new();
    let mut y_grid_lines = String::new();
    let mut current_y_value = y_start;
    while current_y_value <= y_end {
        let y_pos = margin_top + chart_height
            - ((current_y_value - min_intensity) / intensity_range) * chart_height;

        // Y-axis label
        y_labels.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" font-family=\"Arial, sans-serif\" font-size=\"10\" fill=\"#6c757d\" text-anchor=\"end\">{value}</text>",
            x = margin_left - 5.0,
            y = y_pos + 3.0,
            value = current_y_value as i32
        ));

        // Horizontal grid line
        y_grid_lines.push_str(&format!(
            "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"#e9ecef\" stroke-width=\"1\"/>",
            x1 = margin_left,
            y1 = y_pos,
            x2 = margin_left + chart_width,
            y2 = y_pos
        ));

        current_y_value += y_step;
    }

    // Generate X-axis markers every 2 hours (8 points since we have 48 points over 24 hours)
    let mut x_labels = String::new();
    let mut x_grid_lines = String::new();
    let _hours_per_point = 0.5; // 30-minute intervals
    let now = chrono::Utc::now();
    let twelve_hours_ago = now - chrono::Duration::hours(12);

    for i in (0..timeline_points.len()).step_by(4) {
        // Every 4 points = 2 hours
        let x_pos = margin_left + (i as f64 / (timeline_points.len() - 1) as f64) * chart_width;
        let time_offset = twelve_hours_ago + chrono::Duration::minutes((i as f64 * 30.0) as i64);
        let time_label = time_offset.format("%H:%M").to_string();

        // X-axis label
        x_labels.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" font-family=\"Arial, sans-serif\" font-size=\"9\" fill=\"#6c757d\" text-anchor=\"middle\">{time_label}</text>",
            x = x_pos,
            y = height - 5.0,
            time_label = time_label
        ));

        // Vertical grid line
        x_grid_lines.push_str(&format!(
            "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"#e9ecef\" stroke-width=\"1\" opacity=\"0.5\"/>",
            x1 = x_pos,
            y1 = margin_top,
            x2 = x_pos,
            y2 = margin_top + chart_height
        ));
    }

    format!(
        "<svg width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">
            <!-- Background -->
            <rect x=\"0\" y=\"0\" width=\"{width}\" height=\"{height}\" fill=\"#f8f9fa\" rx=\"5\"/>
            
            <!-- Chart area -->
            <rect x=\"{chart_x}\" y=\"{chart_y}\" width=\"{chart_width}\" height=\"{chart_height}\" fill=\"white\" stroke=\"#dee2e6\" stroke-width=\"1\"/>
            
            <!-- Grid lines -->
            {y_grid_lines}
            {x_grid_lines}
            
            <!-- Historical data -->
            <path d=\"{path_data}\" stroke=\"#2c3e50\" stroke-width=\"2\" fill=\"none\"/>
            
            <!-- Forecast data -->
            <path d=\"{forecast_path_data}\" stroke=\"#7f8c8d\" stroke-width=\"2\" fill=\"none\" stroke-dasharray=\"5,5\"/>
            
            <!-- Current time marker -->
            <line x1=\"{current_x}\" y1=\"{marker_y1}\" x2=\"{current_x}\" y2=\"{marker_y2}\" stroke=\"#e74c3c\" stroke-width=\"2\"/>
            
            <!-- Y-axis labels -->
            {y_labels}
            
            <!-- X-axis labels -->
            {x_labels}
            
            <!-- Axis labels -->
            <text x=\"{time_label_x}\" y=\"{time_label_y}\" font-family=\"Arial, sans-serif\" font-size=\"11\" fill=\"#495057\" text-anchor=\"middle\">Time</text>
            <text x=\"{y_axis_label_x}\" y=\"{y_axis_label_y}\" font-family=\"Arial, sans-serif\" font-size=\"11\" fill=\"#495057\" text-anchor=\"middle\" transform=\"rotate(-90 {y_axis_label_x} {y_axis_label_y})\">gCO₂/kWh</text>
        </svg>",
        width = width,
        height = height,
        chart_x = margin_left,
        chart_y = margin_top,
        chart_width = chart_width,
        chart_height = chart_height,
        y_grid_lines = y_grid_lines,
        x_grid_lines = x_grid_lines,
        path_data = path_data,
        forecast_path_data = forecast_path_data,
        current_x = current_x,
        marker_y1 = margin_top,
        marker_y2 = margin_top + chart_height,
        y_labels = y_labels,
        x_labels = x_labels,
        time_label_x = width / 2.0,
        time_label_y = height - 15.0,
        y_axis_label_x = 15.0,
        y_axis_label_y = height / 2.0
    )
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(serve_app))
        .layer(ServiceBuilder::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{addr}", addr = addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

