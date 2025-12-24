# Caelum - Network I/O

## Overview

Network operations for HTTP, WebSocket, and raw sockets. The "sky" to `solum`'s "ground" — remote I/O over the network.

## Etymology

- `caelum` - "sky, heavens" — remote/cloud operations
- `pete` - "seek, request" — HTTP request
- `mitte` - "send" — send data
- `accipe` - "receive" — receive data
- `iunge` - "join, connect" — establish connection
- `finde` - "split, disconnect" — close connection
- `ausculta` - "listen" — server listening

---

## HTTP

### Basic Requests

```
ex "norma/caelum" importa { pete }

// GET request (explicit cede)
fixum response = cede pete("https://api.example.com/users")

// GET request (figendum - implicit await, more natural Latin)
figendum response = pete("https://api.example.com/users")

// With options
figendum response = pete("https://api.example.com/users", {
    modus: "POST",
    corpus: { nomen: "Marcus" },
    capita: { "Content-Type": "application/json" }
})
```

### Request Options

```
{
    modus: "GET" | "POST" | "PUT" | "DELETE" | "PATCH",
    corpus: object | textus | octeti,    // request body
    capita: tabula<textus, textus>,      // headers
    tempus: numerus,                      // timeout in ms
    sequere: bivalens,                    // follow redirects (default: verum)
}
```

### Response Object

```
genus Response {
    status: numerus           // 200, 404, etc.
    statusTextus: textus      // "OK", "Not Found"
    capita: tabula<textus, textus>

    functio corpus() fiet octeti
    functio textus() fiet textus
    functio json() fiet object
}
```

### Convenience Methods

```
ex "norma/caelum" importa { pete, mitte, pone, dele }

// GET
fixum data = cede pete(url).json()

// POST
fixum result = cede mitte(url, { nomen: "Marcus" })

// PUT
fixum result = cede pone(url, updated_data)

// DELETE
cede dele(url)
```

---

## WebSocket

### Client

```
ex "norma/caelum" importa { iunge }

cura iunge("wss://echo.example.com") fit ws {
    // Send message
    ws.mitte("Salve!")

    // Receive messages
    ex ausculta ws.accipe() fiet msg {
        scribe "Received:", msg
    }
}
// ws closed
```

### With Events

```
cura iunge(url) fit ws {
    ws.aperi(fac fit {
        scribe "Connected"
    })

    ws.erratum(fac err fit {
        mone "Error:", err
    })

    ws.claude(fac fit {
        scribe "Disconnected"
    })

    ex ausculta ws.accipe() fiet msg {
        process(msg)
    }
}
```

---

## Raw Sockets

### TCP Client

```
ex "norma/caelum" importa { socket }

cura socket("tcp", "example.com", 80) fit sock {
    sock.mitte("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
    fixum response = sock.accipe(4096)
}
// sock closed
```

### TCP Server

```
ex "norma/caelum" importa { ausculta }

fixum server = ausculta("tcp", "0.0.0.0", 8080)

ex ausculta server.accepta() fiet client {
    cura client fit conn {
        fixum request = conn.accipe(1024)
        conn.mitte("HTTP/1.1 200 OK\r\n\r\nSalve!")
    }
}
```

### UDP

```
ex "norma/caelum" importa { socket }

cura socket("udp", "0.0.0.0", 5000) fit sock {
    // Send
    sock.mitte_ad("message", "192.168.1.100", 5001)

    // Receive
    fixum (data, sender) = sock.accipe_ex(1024)
    scribe "From:", sender.address
}
```

---

## DNS

```
ex "norma/caelum" importa { resolve }

fixum addresses = cede resolve("example.com")
// ["93.184.216.34"]

fixum records = cede resolve("example.com", "MX")
// [{ priority: 10, exchange: "mail.example.com" }]
```

---

## Target Mappings

### TypeScript (Node.js)

```typescript
// pete(url)
await fetch(url)

// socket("tcp", host, port)
import { createConnection } from 'net';
const socket = createConnection({ host, port });

// ausculta("tcp", host, port)
import { createServer } from 'net';
const server = createServer();
server.listen(port, host);

// WebSocket
import WebSocket from 'ws';
const ws = new WebSocket(url);
```

### Python

```python
import httpx
import socket
import asyncio

# pete(url)
async with httpx.AsyncClient() as client:
    response = await client.get(url)

# socket("tcp", host, port)
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.connect((host, port))

# ausculta("tcp", host, port)
server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.bind((host, port))
server.listen()
```

### Rust

```rust
// pete(url) - using reqwest
let response = reqwest::get(url).await?;

// socket("tcp", host, port)
use std::net::TcpStream;
let stream = TcpStream::connect((host, port))?;

// ausculta("tcp", host, port)
use std::net::TcpListener;
let listener = TcpListener::bind((host, port))?;
```

### Zig

```zig
const std = @import("std");
const net = std.net;

// socket("tcp", host, port)
const stream = try net.tcpConnectToHost(allocator, host, port);
defer stream.close();

// ausculta("tcp", host, port)
var server = try net.StreamServer.init(.{});
try server.listen(address);
```

---

## Error Handling

Network errors use `fors` result type:

```
fixum result = cede pete(url)

si result.status >= 400 {
    iace "HTTP error: " + result.status
}
```

Timeout:

```
fixum response = cede pete(url, { tempus: 5000 }) cape err {
    si err.tempus_excessum {
        mone "Request timed out"
        redde fallback_data
    }
    iace err
}
```

---

## Async Model

All network operations are async by default:

```
futura functio fetchUsers() -> lista<User> {
    fixum response = cede pete("/api/users")
    redde cede response.json()
}
```

For parallel requests:

```
// Using explicit cede
fixum (users, posts) = cede promissum.omnes([
    pete("/api/users").json(),
    pete("/api/posts").json()
])

// Using figendum (implicit await)
figendum (users, posts) = promissum.omnes([
    pete("/api/users").json(),
    pete("/api/posts").json()
])
```

---

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| `pete` (HTTP GET) | Not Done | Basic HTTP client |
| `mitte` (HTTP POST) | Not Done | POST requests |
| `pone` (HTTP PUT) | Not Done | PUT requests |
| `dele` (HTTP DELETE) | Not Done | DELETE requests |
| Response parsing | Not Done | json, textus, octeti |
| WebSocket client | Not Done | iunge, mitte, accipe |
| TCP socket | Not Done | Client and server |
| UDP socket | Not Done | Datagram support |
| DNS resolve | Not Done | Name resolution |
| Timeout handling | Not Done | Request timeouts |
| TypeScript codegen | Not Done | fetch, net, ws |
| Python codegen | Not Done | httpx, socket |
| Rust codegen | Not Done | reqwest, std::net |
| Zig codegen | Not Done | std.net |

---

## Design Decisions

### Why `pete` for requests?

`pete` means "seek, request" — fitting for HTTP requests. Alternatives:
- `roga` ("ask") — too conversational
- `quaere` ("search") — already used for file seek

### Why async by default?

Network I/O is inherently slow and unpredictable. Sync network calls block the entire program. Async is the only sensible default for network operations.

### Why `cura` for connections?

Sockets and WebSockets need cleanup. Using `cura` ensures connections are closed on scope exit, preventing resource leaks.

### HTTP vs raw sockets?

Both provided:
- `pete`/`mitte`/`pone`/`dele` — high-level HTTP (90% of use cases)
- `socket`/`ausculta` — low-level for protocols, games, custom services

### Error handling?

Network errors are recoverable (timeouts, connection refused, HTTP errors). Use `iace` for throwing, `cape` for catching. Fatal network issues (no network interface) can `mori`.
