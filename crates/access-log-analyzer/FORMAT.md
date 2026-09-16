# Access log format (reference)

Common Log Format (Apache/Nginx style), one entry per line:

```
127.0.0.1 - - [10/Sep/2026:13:55:36 +0000] "GET /api/users HTTP/1.1" 200 1234
```

| Field | Example | Meaning |
|---|---|---|
| Client IP | `127.0.0.1` | who made the request |
| Identd | `-` | almost always `-` (unused, legacy) |
| User ID | `-` | almost always `-` (unused unless auth is logged) |
| Timestamp | `[10/Sep/2026:13:55:36 +0000]` | when the request was received |
| Request | `"GET /api/users HTTP/1.1"` | method, path, HTTP version |
| Status | `200` | HTTP status code |
| Size | `1234` | response body size in bytes, or `-` if none |

Fields are space-separated, except the timestamp (`[...]`) and the request
(`"..."`), which are themselves space-separated internally but wrapped in
brackets/quotes precisely so they can contain spaces without ambiguity.
