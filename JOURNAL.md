# Vibe Coded Carbon Intensity Tools

## PoC

- We memorized the API URL:
https://carbon-intensity.github.io/api-definitions/#carbon-intensity-api-v2-0-0
- Create a CLI to fetch the current cabon intensity from the UK Carbon
Intensity API.
- Add trace logging for all API interactions
  - Needed to ask to set default log level to INFO and not log library trace
  statements.
- Added a second CLI to show the hourly intensity history for the last 12
hours.
  - As the API has 30 minute intervals, it showed all results. Asked it to
  combine the readings to create 12 hourly readings.
  - I asked it to explain how it combined the readings into one hourly reading
  (it was averaging them).
- Create a leptos web app to display the current intensity and a pie chart with
the energy source breakdown, showing per-source carbon intensity levels.
  - The initial version does not work: acceptable layout but the data is
  missing.
  - It fixed the problem.
- Asked for labels on each segment, arrayed around the outside of the pie, with
lines indicating which segment that refer to.
  - Some of the labels are cut off... asked to remove the lines and place the
  labels closer.
    - This has worked.
- Updated the pie chart to show the carbon intensity of each source.
- ASIDE: I'd like to get CC to keep a transcript of the interactions with the
project in a log file in the project... need to ask how to do that next.
  - It does not have this feature, but can help you manually create one.
  - You can ask it to create a DEVELOPMENT log showing what it's done.
  - I asked CC to always update DEVELOPMENT LOG with for changes in the
  project... it said it would... not sure how it will remember that.
  - DONE Check this is happening without being asked.
    - It remembered to update the dev log...
      - DONE Add this to the CLAUDE.md asking to keep the file up to do date
      with all changes: "# Remember to update the DEVELOPMENT LOG with all
      project changes"
- Added a carbon intensity history and forecast graph.
  - Needed to ask for x and y axis and labels.
- Asked CC to fix all warnings. It used clippy and fixed the resulting
suggestions.
- Asked CC to replace the positional params in the format! calls with named
parameters... it looks very brittle at the moment!
- These two code improvement changes did not trigger development log updates...
did CC forget or does it assume these don't need logging?
