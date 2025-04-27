# Project Structure

This document provides a high-level overview of the directory structure for the
Systeminit/si TUI Client project.

- **`.cargo/`**: Contains Cargo build configuration (`config.toml`).
- **`.github/`**: (Likely for GitHub Actions workflows - currently not present).
- **`docs/`**: Contains project documentation, like this file.
- **`plans/`**: Contains planning documents for development tasks.
- **`prompts/`**: Stores prompts given to the AI assistant during development.
- **`specs/`**: Contains specification documents:
  - `api_specification.md`: Details the server-side API endpoints.
  - `system_specification.txt`: Describes the TUI application's architecture,
    features, and implementation details.
- **`src/`**: Contains the Rust source code:
  - `main.rs`: The main binary entry point. Sets up the terminal and runs the
    application loop.
  - `lib.rs`: The library entry point, declaring core modules.
  - `app.rs`: Defines the main application state (`App` struct).
  - `api_models.rs`: Defines data structures (structs/enums) for API
    request/response bodies using `serde`.
  - `api_client/`: Module containing functions for making specific API calls to
    the server. Each endpoint typically has its own file (e.g.,
    `list_change_sets.rs`).
  - `run_app/`: Module containing the main application loop (`run_app.rs`) and
    event handling logic (`event_handler.rs`).
  - `ui/`: Module containing UI rendering helper functions (e.g.,
    `render_top_bar.rs`, `render_log_panel.rs`).
  - `ui.rs`: Defines the main UI rendering function that constructs the layout
    using `ratatui`.
  - `refresh_change_sets.rs`: Helper function to refresh the list of change
    sets.
- **`target/`**: Default directory for Cargo build artifacts (ignored by Git).
- **`tests/`**: Contains automated tests:
  - `api/`: Integration tests for the API client functions.
  - `unit/`: Unit tests for specific modules or functions (e.g., state
    management, model deserialization).
- **`.env.example`**: (Optional) Example environment file structure. A `.env`
  file (ignored by Git) is required to run the application, containing `SI_API`
  and `JWT_TOKEN`.
- **`.gitignore`**: Specifies intentionally untracked files that Git should
  ignore.
- **`Cargo.toml`**: The Cargo manifest file, defining project metadata and
  dependencies.
- **`Cargo.lock`**: Records the exact versions of dependencies used.
- **`openapi.json`**: OpenAPI schema definition for the Systeminit/si API.
- **`README.md`**: The main introductory document for the project.
- **`rust-toolchain.toml`**: Specifies the Rust toolchain channel (e.g.,
  nightly).
- **`rustfmt.toml`**: Configuration file for `rustfmt` code formatting.
- **`situation.cast`**: (Likely an asciinema recording - specific to this
  environment).
