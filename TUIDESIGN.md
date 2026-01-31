The TUI will provide a simple way to watch logs, spans and events streamed to the console. It should:

- Show each item one line at a time.
- Always follow the latest records, let old records fall off the top.
- Show records in color, with timestamps dimmed, level names colored, target dimmed, messages in regular text, and display attributes inline, skipping the `code.*` attributes and other attributes already displayed. This should feel and look similar to the way tracing_subscriber prints oneline output.
- Include a text input bar at the bottom of the TUI which accepts a string in a format similar to tracing_subscriber's log configuration variables, e.g. `target=level` (no need to support span attributes or advanced env filter details)
- Include a metadata bar at the top of the screen - and show the time since last update, changing color to yellow and red if we fall too far behind.
- Quit on signint
