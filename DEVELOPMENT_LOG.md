# Development Session Log

**Date**: 2025-06-26  
**Project**: carbon-vibe - UK Carbon Intensity CLI Tools and Web Dashboard

## Session Overview
This session focused on building a comprehensive set of tools for accessing and visualizing UK carbon intensity data using the Carbon Intensity API.

## Tasks Completed

### 1. Project Initialization and Structure
- **Created CLAUDE.md**: Initial project documentation for future Claude Code sessions
- **Set up Rust project**: Using Cargo with 2024 edition
- **Configured dependencies**: Added HTTP client, JSON parsing, async runtime, logging, datetime handling, and web framework dependencies

### 2. CLI Tool: `current`
**Location**: `src/bin/current.rs`  
**Purpose**: Display current UK carbon intensity as a single value

**Features**:
- Fetches real-time data from `https://api.carbonintensity.org.uk/intensity`
- Returns just the carbon intensity number (e.g., "102")
- Comprehensive trace-level logging for API interactions
- Environment-controlled logging (`RUST_LOG=trace` for detailed output)

**Data Structures**:
```rust
struct CarbonIntensityData {
    data: Vec<CarbonIntensityEntry>,
}

struct CarbonIntensityEntry {
    intensity: IntensityData,
}

struct IntensityData {
    actual: i32,
}
```

**Usage**:
- `cargo run --bin current` - Clean output with just the intensity value
- `RUST_LOG=trace cargo run --bin current` - Detailed API logging

### 3. CLI Tool: `history`
**Location**: `src/bin/history.rs`  
**Purpose**: Show carbon intensity history for the last 12 hours with hourly aggregation

**Features**:
- Fetches 12-hour historical data using date range API endpoint
- Combines 30-minute API intervals into hourly averages
- Displays 12 lines of hourly data in chronological order
- Proper datetime parsing and formatting
- Uses actual values when available, falls back to forecast

**Key Implementation**:
- Groups data by hour using `BTreeMap<String, Vec<i32>>`
- Calculates averages: `sum / count` for each hour
- Date range construction: `now - 12 hours` to `now`

**Output Format**:
```
2025-06-25 20:00: 187
2025-06-25 21:00: 180
...
2025-06-26 07:00: 92
```

**Usage**:
- `cargo run --bin history` - 12 hourly readings
- `RUST_LOG=trace cargo run --bin history` - With API logging

### 4. Web Application: `web`
**Location**: `src/bin/web.rs`  
**Purpose**: Web dashboard showing current intensity and energy generation mix visualization

**Features**:
- **Current Intensity Display**: Large, prominent display of current carbon intensity
- **Pie Chart Visualization**: SVG-based pie chart showing energy source breakdown
- **Color-coded Legend**: Shows each energy source with percentage and color
- **Responsive Design**: Grid layout with modern styling
- **Server-side Rendering**: Uses Axum web framework

**Data Sources**:
- Current intensity: `https://api.carbonintensity.org.uk/intensity`
- Generation mix: `https://api.carbonintensity.org.uk/generation`

**Technical Implementation**:
- **Pie Chart Math**: Calculates angles and SVG paths for each energy source
- **Color Palette**: 15 distinct colors for different fuel types
- **Data Processing**: Handles percentages and creates proportional chart segments

**Styling**: Clean, modern design with:
- White cards on light gray background
- Professional color scheme (#2c3e50, #7f8c8d)
- Grid layout for desktop viewing
- Box shadows and rounded corners

**Usage**:
- `cargo run --bin web`
- Access at: `http://127.0.0.1:3000`

## Dependencies Added
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = { version = "0.4", features = ["serde"] }
leptos = { version = "0.6", features = ["csr"] }
leptos_axum = "0.6"
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["fs"] }
wasm-bindgen = "0.2"
```

## Project Structure
```
carbon-vibe/
├── Cargo.toml
├── CLAUDE.md
├── DEVELOPMENT_LOG.md
└── src/
    └── bin/
        ├── current.rs    # Current intensity CLI
        ├── history.rs    # 12-hour history CLI
        └── web.rs        # Web dashboard
```

## API Endpoints Used
1. **Current Intensity**: `GET https://api.carbonintensity.org.uk/intensity`
2. **Historical Data**: `GET https://api.carbonintensity.org.uk/intensity/{from}/{to}`
3. **Generation Mix**: `GET https://api.carbonintensity.org.uk/generation`

## Key Technical Decisions

### Logging Strategy
- **Default**: INFO level for clean output
- **Trace Mode**: `RUST_LOG=trace` shows detailed API interactions
- **Filtered Logging**: Only shows application traces, not library noise
- **Per-Binary Configuration**: Each binary has its own logging filter

### Data Processing
- **Current Tool**: Simple passthrough of API data
- **History Tool**: Aggregates 30-minute intervals into hourly averages using BTreeMap
- **Web Tool**: Server-side rendering for immediate display

### Error Handling
- Graceful fallbacks (actual → forecast → 0)
- Proper error propagation with `?` operator
- User-friendly error messages for datetime parsing

## Testing Results
All three applications were successfully tested:
- **current**: Returns single intensity value (e.g., "87")
- **history**: Shows 12 hourly readings in chronological order
- **web**: Serves dashboard at localhost:3000 with live data

## Future Enhancements
Potential improvements identified during development:
1. **Caching**: Add response caching to reduce API calls
2. **Real-time Updates**: WebSocket updates for the web dashboard
3. **Historical Charts**: Line graphs for trend visualization
4. **Regional Data**: Support for regional carbon intensity
5. **Alerts**: Threshold-based notifications for high/low intensity
6. **Export**: CSV/JSON export functionality

## Session Notes
- Moved from main.rs to bin/ structure for multiple executables
- Simplified Leptos implementation to Axum for better reliability
- Fixed compiler warnings and datetime parsing issues
- Implemented mathematical pie chart generation with SVG
- Added comprehensive trace logging for debugging and transparency

## Bug Fixes and Debugging

### Web Application Issues (Resolved)
**Problem**: Web application displayed "0" for carbon intensity and no pie chart visualization.

**Root Cause**: Incorrect data structure for Generation Mix API response
- Expected: `data: Vec<GenerationMixEntry>` (array)
- Actual API response: `data: GenerationMixEntry` (single object)

**API Response Analysis**:
```bash
# Carbon Intensity API (working correctly)
curl "https://api.carbonintensity.org.uk/intensity"
# Returns: {"data": [{"intensity": {"actual": 74, "forecast": 92}}]}

# Generation Mix API (structure issue)
curl "https://api.carbonintensity.org.uk/generation"  
# Returns: {"data": {"generationmix": [{"fuel": "biomass", "perc": 9.3}, ...]}}
```

**Fixes Applied**:
1. **Data Structure Correction**:
   ```rust
   // Before (incorrect)
   struct GenerationMixData {
       data: Vec<GenerationMixEntry>,
   }
   
   // After (correct)
   struct GenerationMixData {
       data: GenerationMixEntry,
   }
   ```

2. **Parsing Logic Update**:
   ```rust
   // Before (incorrect - trying to get first element of non-array)
   let generation_mix = mix_data.data.first()
       .map(|entry| entry.generation_mix.clone())
       .unwrap_or_default();
   
   // After (correct - direct access to object)
   let generation_mix = mix_data.data.generation_mix;
   ```

3. **Added Debug Logging**:
   ```rust
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
   ```

**Verification Results**:
- ✅ Carbon Intensity: Now displays actual value (e.g., "74 gCO₂/kWh")
- ✅ Pie Chart: Renders colorful SVG with proper proportions
- ✅ Legend: Shows all 9 fuel sources with percentages
- ✅ Data Fetching: Console shows "Successfully fetched data: intensity=74, mix_items=9"

**Current Fuel Mix Example**:
- Wind: 34.4% (largest segment)
- Gas: 20.9%
- Nuclear: 15.9% 
- Imports: 10.7%
- Biomass: 9.3%
- Solar: 9.4%
- Coal: 0.0%
- Hydro: 0.0%
- Other: 0.0%

The web application is now fully functional and accurately displays live UK carbon intensity data with visual breakdown of energy sources.

## Enhanced Pie Chart Visualization

### External Labels with Connecting Lines (Latest Update)
**Enhancement**: Added external labels with connecting lines to the pie chart for better readability.

**Changes Made**:
1. **Expanded SVG Canvas**: Increased from 400x400 to 500x500 to accommodate external labels
2. **Centered Pie Chart**: Updated coordinates to center the pie (250,250) in the larger canvas
3. **External Label Positioning**: 
   - Labels positioned at radius 220px from center
   - Connecting lines start at radius 155px (just outside pie)
   - Smart text anchoring: `end` for left side labels, `start` for right side labels
4. **Label Filtering**: Only show labels for segments ≥0.5% to avoid clutter
5. **Label Content**: Display both fuel name and percentage (e.g., "wind (35.0%)")

**Technical Implementation**:
```rust
// Calculate label position (middle of arc, extended outward)
let mid_angle = start_angle + angle / 2.0;
let label_radius = 220.0; // Distance from center for label
let line_start_radius = 155.0; // Start line just outside pie

// Smart text anchoring based on position
let text_anchor = if mid_angle > π/2 && mid_angle < 3π/2 {
    "end"  // Left side of chart
} else {
    "start"  // Right side of chart
};
```

**Visual Improvements**:
- ✅ **Clear Labels**: Each significant energy source clearly labeled outside the pie
- ✅ **Connecting Lines**: Gray lines (#666666) connect labels to their pie segments
- ✅ **Percentage Display**: Shows both name and percentage for each source
- ✅ **Clean Layout**: Small segments (<0.5%) excluded from labels to prevent clutter
- ✅ **Proper Alignment**: Text anchored intelligently based on position

**Current Example Data Visualization**:
- Wind: 35.0% (largest segment, top-right)
- Gas: 16.3% (bottom-left)
- Nuclear: 16.1% (left side)
- Solar: 12.1% (top-left)
- Imports: 11.2% (bottom-right)
- Biomass: 9.2% (right side)

The enhanced pie chart now provides a professional, easy-to-read visualization of the UK's energy generation mix with clear labeling and connecting lines for each significant energy source.

### Label Positioning Refinement (Latest Update)
**Issue**: Some external labels were being cut off at the panel edges.

**Solution**: Repositioned labels closer to pie segments and removed connecting lines.

**Changes Made**:
1. **Closer Label Positioning**: Moved labels from radius 220px to 175px (closer to pie edge)
2. **Removed Connecting Lines**: Eliminated gray connecting lines to simplify the design
3. **Two-Line Labels**: Split labels into fuel name (bold, 11px) and percentage (lighter, 10px)
4. **Center Alignment**: All labels now use "middle" text anchor for consistent alignment
5. **Reduced Canvas**: Adjusted SVG display size to 450x450px while keeping 500x500 viewBox

**Technical Implementation**:
```rust
// Simplified label positioning - closer to pie edge
let label_radius = 175.0; // Closer to the pie edge
let label_x = center_x + label_radius * mid_angle.cos();
let label_y = center_y + label_radius * mid_angle.sin();

// Two-line labels with different styling
elements.push_str(&format!(
    "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"11\" font-weight=\"bold\">{}</text>",
    label_x, label_y - 2.0, fuel.fuel
));
elements.push_str(&format!(
    "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#666666\">{:.1}%</text>",
    label_x, label_y + 10.0, fuel.perc
));
```

**Final Visual Result**:
- ✅ **No Cut-off Labels**: All labels now fit within the chart boundaries
- ✅ **Clean Design**: Removed connecting lines for simpler appearance
- ✅ **Better Readability**: Two-line format with fuel name and percentage
- ✅ **Consistent Alignment**: All labels center-aligned for uniformity
- ✅ **Appropriate Sizing**: Labels positioned just outside pie segments

The pie chart now provides an optimal balance between information density and visual clarity, with all labels clearly visible within the chart boundaries.

## Carbon Intensity Factors Integration

### Enhanced Legend with Per-Fuel Carbon Intensities (Latest Update)
**Enhancement**: Added carbon intensity factors for each energy source to provide deeper insights into environmental impact.

**API Integration**:
- **New Endpoint**: `https://api.carbonintensity.org.uk/intensity/factors`
- **Data Source**: Official carbon intensity factors for each fuel type in gCO₂/kWh

**Implementation Details**:

1. **New Data Structures**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CarbonFactorsData {
    data: Vec<CarbonFactors>,
}

#[derive(Clone, Debug)]
struct FuelSourceWithIntensity {
    fuel: String,
    perc: f64,
    carbon_intensity: i32,
}
```

2. **Enhanced API Calls**: Now fetches three data sources concurrently:
   - Current intensity: `/intensity`
   - Generation mix: `/generation`
   - Carbon factors: `/intensity/factors`

3. **Fuel Type Mapping**: Intelligent mapping between generation mix names and factor names:
```rust
let carbon_intensity = match fuel.fuel.as_str() {
    "biomass" => factors.biomass,           // 120 gCO₂/kWh
    "coal" => factors.coal,                 // 937 gCO₂/kWh
    "gas" => factors.gas_combined_cycle,    // 394 gCO₂/kWh
    "nuclear" => factors.nuclear,           // 0 gCO₂/kWh
    "solar" => factors.solar,               // 0 gCO₂/kWh
    "wind" => factors.wind,                 // 0 gCO₂/kWh
    "imports" => average_import_factor,     // 328 gCO₂/kWh (calculated)
    _ => 0,
};
```

4. **Enhanced Legend Design**:
   - **Two-line format**: Fuel name (bold) + percentage and carbon intensity (subtitle)
   - **Improved spacing**: Better grid layout with increased gaps
   - **Visual hierarchy**: Clear distinction between fuel name and details

**Carbon Intensity Insights**:
- ✅ **Clean Energy Sources**: Wind (0), Solar (0), Nuclear (0), Hydro (0)
- ✅ **Low Emission Sources**: Biomass (120 gCO₂/kWh)
- ✅ **Medium Emission Sources**: Gas (394 gCO₂/kWh), Imports (328 gCO₂/kWh avg)
- ✅ **High Emission Sources**: Coal (937 gCO₂/kWh), Oil (935 gCO₂/kWh)

**Example Current UK Mix** (showing environmental impact):
- Wind: 45.7% • 0 gCO₂/kWh (largest clean contribution)
- Solar: 28.0% • 0 gCO₂/kWh (significant clean energy)
- Nuclear: 13.3% • 0 gCO₂/kWh (stable clean baseload)
- Gas: 5.7% • 394 gCO₂/kWh (moderate emissions for backup)
- Imports: 5.1% • 328 gCO₂/kWh (mixed European grid)
- Biomass: 2.2% • 120 gCO₂/kWh (renewable but not zero-carbon)

**Environmental Benefits**:
The enhanced visualization now shows that the UK's current energy mix is dominated by clean sources (86.0% from wind, solar, and nuclear combined), with minimal contribution from high-carbon sources. This provides users with immediate insight into both the composition and environmental impact of their electricity supply.

**Technical Achievements**:
- ✅ **Multi-API Integration**: Seamlessly combines three different API endpoints
- ✅ **Intelligent Data Mapping**: Handles naming differences between APIs
- ✅ **Enhanced UX**: Users can now see both generation mix and environmental impact
- ✅ **Real-time Insights**: Live data showing the carbon footprint of current electricity
- ✅ **Educational Value**: Immediate understanding of which energy sources are cleanest

## 24-Hour Intensity Graph Integration (Latest Update)

### Enhanced Current Carbon Intensity Pane
**Enhancement**: Added a 24-hour intensity graph showing 12 hours of historical data and 12 hours of forecast data to the Current Carbon Intensity pane.

**Implementation Details**:

1. **Extended Data Fetching**: Modified `fetch_carbon_data()` to include timeline data:
   ```rust
   // Fetch 24-hour timeline data (12 hours past + 12 hours future)
   let now = chrono::Utc::now();
   let twelve_hours_ago = now - chrono::Duration::hours(12);
   let twelve_hours_future = now + chrono::Duration::hours(12);
   
   let timeline_url = format!(
       "https://api.carbonintensity.org.uk/intensity/{}/{}",
       from_date, to_date
   );
   ```

2. **Data Processing**: Converts API response into `IntensityPoint` structs:
   ```rust
   struct IntensityPoint {
       datetime: String,
       intensity: i32,
       is_forecast: bool,
   }
   ```

3. **SVG Line Chart**: Created `render_intensity_chart()` function with:
   - **Historical Data**: Solid line showing past 12 hours
   - **Forecast Data**: Dashed line showing next 12 hours  
   - **Current Time Marker**: Red vertical line indicating "now"
   - **Grid Lines**: Horizontal guidelines for readability
   - **Time Labels**: "12h ago", "now", "12h future" markers

4. **Chart Features**:
   - **Dynamic Scaling**: Automatically scales Y-axis based on data range
   - **Visual Distinction**: Historical (solid) vs forecast (dashed) lines
   - **Responsive Design**: 400x120px chart fits well in dashboard layout
   - **Error Handling**: Gracefully handles empty data or single data points

**Technical Implementation**:
```rust
fn render_intensity_chart(timeline_points: &[IntensityPoint]) -> String {
    // Calculate scaling for intensity values
    let min_intensity = *intensities.iter().min().unwrap_or(&0) as f64;
    let max_intensity = *intensities.iter().max().unwrap_or(&100) as f64;
    
    // Generate SVG path data for line chart
    for (i, point) in timeline_points.iter().enumerate() {
        let x = margin + (i as f64 / (timeline_points.len() - 1) as f64) * chart_width;
        let y = margin + chart_height - ((point.intensity as f64 - min_intensity) / intensity_range) * chart_height;
        
        if point.is_forecast {
            forecast_path_data.push_str(&format!(" L {} {}", x, y));
        } else {
            path_data.push_str(&format!(" L {} {}", x, y));
        }
    }
}
```

**Data Structure Updates**:
- Modified `CarbonIntensityEntry` to include optional `from` and `to` fields for timeline data
- Enhanced `fetch_carbon_data()` return type to include `Vec<IntensityPoint>`
- Updated HTML template to include chart in intensity display pane

**Visual Integration**:
- **Seamless Layout**: Chart positioned below current intensity value
- **Consistent Styling**: Matches existing dashboard color scheme
- **Informative Display**: Shows trends and patterns in carbon intensity over time

**API Data Usage**:
- **48 Data Points**: 30-minute intervals over 24 hours (12h past + 12h future)
- **Smart Fallbacks**: Uses actual values when available, forecast otherwise
- **Current Time Detection**: Identifies transition point between historical and forecast data

**Current Example**:
The graph shows intensity ranging from 35-150 gCO₂/kWh over the 24-hour period, with clear visual distinction between historical solid line and forecast dashed line, separated by a red "now" marker.

**Benefits for Users**:
- ✅ **Trend Visualization**: See how carbon intensity changes throughout the day
- ✅ **Planning Tool**: Use forecast data to plan energy-intensive activities
- ✅ **Historical Context**: Understand recent patterns and variations
- ✅ **Real-time Awareness**: Current position clearly marked on timeline
- ✅ **Data Confidence**: Visual distinction between actual and forecast data

The enhanced dashboard now provides comprehensive carbon intensity insights combining current values, energy source breakdown with environmental impact, and 24-hour trend visualization in a single, cohesive interface.

## Code Quality Improvements (Latest Updates)

### Compile Warning Fixes
**Enhancement**: Fixed all compile warnings identified by Clippy and Rust compiler.

**Warnings Resolved**:

1. **Collapsible else-if Block** (`src/bin/web.rs`):
   ```rust
   // Before: nested else-if structure
   } else {
       if point.is_forecast {
           // ... logic
       }
   }
   
   // After: collapsed structure
   } else if point.is_forecast {
       // ... logic
   }
   ```

2. **Unnecessary or_insert_with** (`src/bin/history.rs`):
   ```rust
   // Before: manual Vec::new construction
   hourly_data.entry(hour_key).or_insert_with(Vec::new).push(intensity);
   
   // After: using or_default()
   hourly_data.entry(hour_key).or_default().push(intensity);
   ```

3. **Unused Variable Warning**: Fixed `_hours_per_point` variable naming in chart generation.

**Result**: ✅ **Zero Compile Warnings** - Clean build across all targets with `cargo clippy --all-targets`

### Enhanced Axis Labels for Intensity Graph
**Enhancement**: Added professional Y-axis and X-axis labels to the 24-hour intensity graph for improved data interpretation.

**Y-Axis Improvements**:
- **Carbon Intensity Labels**: Shows gCO₂/kWh values with dynamic scaling
- **Smart Stepping**: Automatically calculates appropriate intervals (minimum 20 units)
- **Grid Lines**: Horizontal reference lines for easy value reading
- **Right-aligned Labels**: Positioned 5px left of chart area for clean appearance

**X-Axis Improvements**:
- **Time Markers**: Shows actual timestamps every 2 hours (e.g., "14:30", "16:30")
- **Real-time Calculation**: Based on actual data timeline (12h past + 12h future)
- **Vertical Grid Lines**: Semi-transparent guides for temporal reference
- **Center-aligned Labels**: Positioned below chart with proper spacing

**Layout Enhancements**:
- **Larger Chart**: Increased to 500x180px to accommodate axis labels
- **Professional Margins**: 50px left, 40px bottom for proper label spacing
- **Chart Background**: White area with border for clear data presentation
- **Axis Titles**: "Time" (bottom) and "gCO₂/kWh" (left, rotated)

**Technical Implementation**:
```rust
// Y-axis labels with dynamic scaling
let y_step = ((max_intensity - min_intensity) / 4.0).ceil().max(20.0);

// X-axis markers every 2 hours (4 data points = 2 hours)
for i in (0..timeline_points.len()).step_by(4) {
    let time_offset = twelve_hours_ago + chrono::Duration::minutes((i as f64 * 30.0) as i64);
    let time_label = time_offset.format("%H:%M").to_string();
}
```

**User Benefits**:
- ✅ **Easy Value Reading**: Y-axis shows exact carbon intensity values
- ✅ **Time Context**: X-axis provides clear temporal reference points
- ✅ **Professional Appearance**: Chart looks publication-ready
- ✅ **Data Interpretation**: Users can easily read trends and specific values

### Named Parameters Migration
**Enhancement**: Replaced all positional parameters in `format!` and `println!` calls with named parameters for improved code readability and maintainability.

**Scope of Changes**:
- **Files Updated**: `src/bin/web.rs`, `src/bin/history.rs`, `src/bin/current.rs`
- **Total Conversions**: 20+ format calls converted to named parameters
- **Function Coverage**: All major formatting functions updated

**Key Improvements**:

1. **HTML Template Formatting**:
   ```rust
   // Before: positional parameters
   format!(r#"...{}...{}...{}...{}..."#,
       intensity,
       render_intensity_chart(&timeline_points),
       render_pie_chart(&generation_mix),
       render_legend(&generation_mix)
   )
   
   // After: named parameters
   format!(r#"...{intensity}...{intensity_chart}...{pie_chart}...{legend}..."#,
       intensity = intensity,
       intensity_chart = render_intensity_chart(&timeline_points),
       pie_chart = render_pie_chart(&generation_mix),
       legend = render_legend(&generation_mix)
   )
   ```

2. **Complex SVG Chart Generation**:
   ```rust
   // Before: 20+ positional parameters (error-prone)
   format!("<svg width=\"{}\" height=\"{}\"...{}", width, height, /* many more */)
   
   // After: clear named mapping
   format!("<svg width=\"{width}\" height=\"{height}\"...{path_data}...{forecast_path_data}...",
       width = width,
       height = height,
       chart_x = margin_left,
       chart_y = margin_top,
       path_data = path_data,
       forecast_path_data = forecast_path_data,
       current_x = current_x,
       // ... all parameters clearly named
   )
   ```

3. **API URL Construction**:
   ```rust
   // Before: positional
   format!("https://api.carbonintensity.org.uk/intensity/{}/{}", from_date, to_date)
   
   // After: named
   format!("https://api.carbonintensity.org.uk/intensity/{from_date}/{to_date}",
       from_date = from_date,
       to_date = to_date
   )
   ```

4. **Logging and Console Output**:
   ```rust
   // Before: positional
   println!("{}: {}", hour, avg_intensity);
   
   // After: named
   println!("{hour}: {intensity}", hour = hour, intensity = avg_intensity);
   ```

**Technical Benefits**:
- ✅ **Type Safety**: Reduced risk of parameter order mistakes
- ✅ **Maintainability**: Easy to modify without counting positions
- ✅ **Readability**: Self-documenting parameter purpose
- ✅ **Debugging**: Clearer error messages when parameters don't match
- ✅ **Refactoring**: Safe to reorder parameters in format strings

**Quality Assurance**:
- ✅ **Build Verification**: `cargo build --all-targets` passes cleanly
- ✅ **Functionality Preserved**: All applications work identically
- ✅ **No Performance Impact**: Named parameters compile to identical bytecode
- ✅ **Future-Proof**: Easier to extend and modify format strings

**Code Quality Metrics**:
- **Zero Compile Warnings**: Complete clean build
- **Enhanced Readability**: Significantly improved code clarity
- **Maintainability Score**: Reduced cognitive load for developers
- **Error Resistance**: Eliminated positional parameter mismatches

The codebase now follows modern Rust best practices with clean, self-documenting format strings and professional-grade data visualization with proper axis labeling.