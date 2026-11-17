# Hosting & Networking

Bombeatbyx uses an **authoritative server**: the server runs the simulation and judges every action, clients only display it and send inputs. There are two ways to get a server.

| Mode | Who runs the server | Best for |
| :--- | :--- | :--- |
| **Online** | A dedicated `server` process on a machine everyone can reach | Playing over the internet |
| **LAN** | The host's client starts a `server` process automatically | Playing on the same local network |

## 1. Ports

| Port | Protocol | Used for |
| :--- | :--- | :--- |
| `27300` (up to `27310` if busy) | TCP (WebSocket, path `/ws`) | Game traffic |
| `27315` | UDP broadcast | LAN room discovery |

Allow these ports in the host's firewall. On Windows, accept the prompt the first time the server starts.

## 2. Online: Running a Dedicated Server

From the release archive:

```bash
./server            # listens on 27300 (or the next free port up to 27310)
./server 27305      # or: ./server --port 27305
```

From source:

```bash
cargo run --release -p server -- 27300
```

The server prints the address it actually bound, for example `Listening on ws://localhost:27300/ws`. Opening `http://<host>:27300/` in a browser shows `Bombeatbyx TUI Server is running!`.

### Connecting players to it

1. In **Settings**, select **Server**, press `Enter`, type the address (`192.168.1.20:27300`, `play.example.com:27300` or `ws://host:port/ws`) and press `Enter` again. The default is `127.0.0.1:27300`, the same machine.
2. **Host Game** with mode **Online** creates a room on that server.
3. Other players open **Join Game** on the **Online** tab, press `R` to refresh the list and `Enter` to join, or `P` to type the code of a **Private** room.

### Running the server in Docker

```bash
docker run --rm -p 27300:27300 -v "${PWD}:/app" -w /app rust:1.88 cargo run --release -p server -- 27300
```

## 3. LAN Games

1. The host chooses **Host Game**, mode **LAN**. The client starts a server on the first free port between 27300 and 27310 (the `server` binary must sit next to the client, as in the release archive).
2. **Public** LAN rooms are announced every second on UDP 27315.
3. Other players open **Join Game** and switch to the **LAN** tab with `Q`/`D`. Rooms seen in the last 3 seconds are listed; `Enter` joins.

**Private LAN rooms** are not announced. To join one, set **Settings > Server** to `<host IP>:<port>`, then use `P` in **Join Game** and type the room code.

The LAN server stops when the hosting player leaves or closes the game.

## 4. Rooms

* Up to **8 players** per room. Players joining a game in progress become **spectators** until the next match.
* Only the host can change room settings, start or stop the game, and kick, ban or promote players.
* If the host disconnects, the player with the lowest id becomes host (everyone is notified).
* A match stops when fewer than two active players remain.
* All room settings are re-validated by the server, so a modified client cannot crash it with invalid values.

## 5. Latency and the Beat

Network delay matters a lot in a rhythm game. Bombeatbyx compensates for it:

* The server measures every player's round-trip time (RTT) with a heartbeat every 3 seconds.
* An action is judged at the moment the key was **pressed** (reception time minus half the RTT), compensating up to **150 ms** of one-way delay.
* Each client realigns its rhythm gauge on the server's beat at every beat, so what you see matches what the server judges.
* Your own moves are shown immediately and confirmed (or corrected) by the server a moment later.

Above roughly 300 ms of round-trip time, timing will feel unfair: prefer a server close to all players.

## 6. Troubleshooting

| Symptom | Likely cause and fix |
| :--- | :--- |
| `No server is running at ...` (when hosting **Online**) | Nothing listens at the address of **Settings > Server**. Start a dedicated server there (section 2), or host in **LAN** mode to use your own machine. |
| `Server did not answer within 5 s` | Wrong address or port, server not running, or firewall. Check **Settings > Server**. |
| `Connection lost: server stopped responding` | No message from the server for 12 seconds (network drop or server crash). |
| Online room list stays empty | Press `R` to refresh. Only **Public** online rooms are listed. |
| LAN room not visible | UDP 27315 blocked, players on different subnets, or the room is **Private**. Only one client per machine can listen for LAN rooms. |
| Hosting LAN fails (`Failed to start server binary` or `Server did not answer`) | The `server` executable is not next to the client. Keep both files from the release archive together. |
| `Room is full` | 8 players maximum. |
| `You are banned from this room` | The host banned your IP for this room. |
