# The Agency - Workflow Builder Desktop App

A native desktop application for creating and managing AI agent workflows.

## Overview

The Workflow Builder is a Tauri-based desktop application that provides:
- Visual workflow designer with drag-and-drop node editor
- Persistent workflow storage in SQLite
- Automatic daemon management (spawns daemon if not running)
- Native desktop experience with web-based UI

## Architecture

The desktop app consists of two components:

1. **Agency Daemon** (`agency-daemon`)
   - HTTP server running on `http://127.0.0.1:8080`
   - Provides REST API for workflow management
   - Persists workflows in SQLite database
   - Serves the workflow UI at `/workflow-ui`

2. **Tauri Desktop App** (`workflow-builder`)
   - Native desktop window (macOS, Linux, Windows)
   - Automatically spawns and manages the daemon
   - Loads the workflow UI from the daemon
   - Provides native OS integration

## Quick Start

### Option 1: Build and Run (Development)

```bash
# Make sure the daemon is running (or the app will start it automatically)
cargo run --bin agency-daemon --release

# In a separate terminal, run the desktop app
cargo run --bin workflow-builder --features tauri
```

### Option 2: Build Standalone Desktop App

```bash
# Build the desktop application
cargo tauri build --bin workflow-builder --features tauri

# The app will be in: target/release/bundle/
# On macOS: target/release/bundle/macos/The Agency - Workflow Builder.app
# On Linux: target/release/bundle/appimage/
# On Windows: target/release/bundle/msi/
```

## Usage

1. **Launch the App**
   - Double-click the app icon (if built as bundle)
   - Or run: `cargo run --bin workflow-builder --features tauri`
   - The app will automatically check if the daemon is running
   - If not running, it will spawn the daemon automatically

2. **Create a Workflow**
   - Click "New Workflow" in the toolbar
   - Enter a name and description
   - Drag nodes from the left sidebar onto the canvas
   - Connect nodes by clicking output ports and dragging to input ports

3. **Save and Execute**
   - Workflows are automatically saved to SQLite
   - Click "Save" to ensure all changes are persisted
   - Click "Execute" to run the workflow
   - Results appear in the properties panel

4. **Access from Browser**
   - The workflow UI is also accessible at: `http://127.0.0.1:8080/workflow-ui`
   - Use this for web-based access or development

## Features

### Visual Workflow Editor
- Drag-and-drop node placement
- Visual connection drawing
- Pan and zoom canvas
- Node configuration panel
- Real-time workflow validation

### Node Types
- **Data Processing**: Text splitters, transformers
- **LLM**: Language model interactions, text generation
- **I/O**: File input/output operations
- **Control Flow**: Conditionals, loops, branches

### Persistence
- All workflows stored in SQLite database
- Workflows persist across daemon restarts
- Located in: `data/memory.db`

### Daemon Auto-Management
- Desktop app automatically checks if daemon is running
- Spawns daemon if needed (up to 30 second startup wait)
- Graceful error handling if daemon fails to start

## Troubleshooting

### Daemon Won't Start

If the desktop app can't start the daemon:

```bash
# Start daemon manually
cargo run --bin agency-daemon --release

# Then launch the desktop app
cargo run --bin workflow-builder --features tauri
```

### Port Already in Use

If port 8080 is already in use:

```bash
# Find and kill the process using port 8080
lsof -ti:8080 | xargs kill -9

# Or change the port in config.toml
```

### Workflows Not Loading

1. Check that the daemon is running:
   ```bash
   curl http://127.0.0.1:8080/health
   ```

2. Check the database exists:
   ```bash
   ls -la data/memory.db
   ```

3. Check daemon logs for errors

### Desktop App Won't Build

Make sure you have Tauri dependencies installed:

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**Windows:**
- Install Microsoft Visual Studio C++ Build Tools
- Install WebView2 (usually pre-installed on Windows 11)

## Development

### File Structure

```
src/bin/
├── agency-daemon.rs       # HTTP daemon server
├── workflow-builder.rs    # Tauri desktop app
└── workflow-ui.rs         # Legacy standalone server (deprecated)

static/
└── index.html            # Workflow UI (served by daemon)

src/
├── api.rs                # REST API handlers
├── ui_workflow_storage.rs # SQLite persistence
└── ...                   # Core agent library
```

### API Endpoints

The daemon provides these endpoints:

- `GET /health` - Health check
- `GET /workflow-ui` - Serve workflow UI HTML
- `GET /workflow-ui/nodes` - List available node types
- `GET /workflow-ui/workflows` - List all workflows
- `POST /workflow-ui/workflows` - Create new workflow
- `GET /workflow-ui/workflows/:id` - Get workflow by ID
- `PUT /workflow-ui/workflows/:id` - Update workflow
- `DELETE /workflow-ui/workflows/:id` - Delete workflow
- `POST /workflow-ui/workflows/:id/execute` - Execute workflow

### Extending Node Types

Add new node types in `src/api.rs`:

```rust
pub fn initialize_ui_node_types(app_state: &AppState) {
    let mut types = app_state.ui_node_types.write().await;

    types.insert("my_custom_node".to_string(), UINodeType {
        id: "my_custom_node".to_string(),
        name: "My Custom Node".to_string(),
        category: "Custom".to_string(),
        description: "Does something custom".to_string(),
        inputs: vec![/* ... */],
        outputs: vec![/* ... */],
        config_schema: None,
    });
}
```

## License

MIT
