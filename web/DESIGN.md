This is a minimal svelte-kit starter project. Please use this as scaffolding, but delete / remove unused components.

The web frontend is a frontend for arboretum, a tool for exploring logs and spans, so we want:

- A bar at the top with a field to enter the service name, and then a search field which can accept a target, and a search field that can accpet a message. A second line should allow filtering by level as well as other attributes. The search fields for target, level and service name should provide autocomplete of the known attributes for those values.
- A main view which will display logs and spans as rows.
- A bottom bar which will display minimal metadata about arboretum

For logs:
- Show the timestamp, in the browser's local time
- Show the log level, with colors
- Show the log target
- Show the log message
Clicking on the log row should expand to a detail view which shows the remaining log attributes.

For spans:
- Show the timestamp, in the browser's local time
- Show the span level, with colors
- Show the span target
- Show the span message
- Show the span 
Clicking on the span should show a span tree diagram on top, and then the span attributes below. Clicking on each span element in the tree diagram should show the span attributes.

It should also be possible to promote attributes to the top level (so they are shown in the row with logs and spans)

Styling should be done using bootstrap5, with bootstrap icons where appropriate. The brand color should be a cyan blue, and the general UI should be dark with light text.
