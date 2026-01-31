Arboretum is a web server for recieving and caching opentelemetry traces and logs over short time periods (1-2 hours).

- Written in rust, driven by tokio
- Uses axum to provide webserving
- Uses sqlite or other on disk storage to store traces and logs. Can be configured for the maximum time period and disk space used for storage.
- Has an endpoint that can be used to submit opentelemetry logs and traces
- Has endpoints for returning traces and logs
- Is optimized for handling logs and traces produced by the rust tracing library
- Accepts filtering parametrs on those endpoints to select subsets of logs and traces:
  - Filter on the tracing "target" attribute
  - filter on the level of the log or span
  - filter on the message
  - filter on the service name
  - filter on span ID
- There should be unit tests for this functionality
