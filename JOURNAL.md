# Vibe Coded Carbon Intensity Tools

## PoC

- We memorized the API URL: https://carbon-intensity.github.io/api-definitions/#carbon-intensity-api-v2-0-0
- Create a CLI to fetch the current cabon intensity from the UK Carbon Intensity API.
- Add trace logging for all API interactions
  - Needed to ask to set default log level to INFO and not log library trace statements.
- Added a second CLI to show the hourly intensity history for the last 12 hours.
  - As the API has 30 minute intervals, it showed all results. Asked it to combine the readings to create 12 hourly readings.
  - I asked it to explain how it combined the readings into one hourly reading (it was averaging them).

