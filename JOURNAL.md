# Vibe Coded Carbon Intensity Tools

## PoC

- We memorized the API URL: https://carbon-intensity.github.io/api-definitions/#carbon-intensity-api-v2-0-0
- Create a CLI to fetch the current cabon intensity from the UK Carbon Intensity API.
- Add trace logging for all API interactions
  - Needed to ask to set default log level to INFO and not log library trace statements.
- Added a second CLI to show the hourly intensity history for the last 12 hours.
  - As the API has 30 minute intervals, it showed all results. Asked it to combine the readings to create 12 hourly readings.
  - I asked it to explain how it combined the readings into one hourly reading (it was averaging them).
- Create a leptos web app to display the current intensity and a pie chart with the energy source breakdown, showing per-source carbon intensity levels.
  - The initial version does not work: acceptable layout but the data is missing.
  - It fixed the problem.

- ASIDE: I'd like to get CC to keep a transcript of the interactions with the project in a log file in the project... need to ask how to do that next.
  - It does not have this feature, but can help you manually create one.
  - You can ask it to create a DEVELOPMENT log showing what it's done.
  - I asked CC to always update DEVELOPMENT LOG with for changes in the project... it said it would... not sure how it will remember that.
  - TODO Check this is happening without being asked.
        - Maybe you can add this to the CLAUDE.md asking to keep the file up to do date with all changes?

