# Systeminit/si TUI Client

## Overview

This is a Terminal User Interface (TUI) application built with Rust for
interacting with the Systeminit/si API. It allows users to view, create, delete,
and manage change sets associated with their workspace directly from the
terminal.

## Features

- **View Workspace Info:** Displays the current workspace ID and user email.
- **List Change Sets:** Shows a list of change sets for the workspace,
  color-coded by status (Completed, Failed, InProgress, Abandoned).
- **View Change Set Details:** Displays detailed information and merge status
  for a selected change set.
- **Create Change Sets:** Allows creating new change sets with a user-provided
  name.
- **Delete Change Sets:** Provides functionality to delete existing change sets.
- **Force Apply Change Sets:** Allows forcing the application of a change set.
- **API Interaction:** Communicates with the Systeminit/si API via HTTP
  requests.
- **Log Viewer:** Displays logs from API interactions and application events.

## Setup

1. **Prerequisites:** Ensure you have Rust installed (`cargo`).
2. **Environment Variables:** Create a `.env` file in the project root with the
   following variables:
   ```dotenv
   SI_API=YOUR_API_ENDPOINT_URL
   JWT_TOKEN=YOUR_JWT_TOKEN
   ```
   Replace `YOUR_API_ENDPOINT_URL` with the base URL of the Systeminit/si API
   and `YOUR_JWT_TOKEN` with your authentication token.
3. **Build:** Compile the project using `cargo build`.
4. **Run:** Execute the application using `cargo run`.

## Usage (Keybindings)

- **`q`**: Quit the application.
- **`Tab`**: Switch focus between the "Workspace" and "Change Set" triggers in
  the top bar.
- **`Enter` / `Space`** (on "Change Set" trigger): Open/close the change set
  selection dropdown.
- **`Up Arrow` / `Down Arrow`** (in dropdown): Navigate the change set list.
- **`Enter`** (in dropdown): Select the highlighted change set and close the
  dropdown.
- **`Esc` / `Tab`** (in dropdown): Close the dropdown without changing
  selection.
- **`c`**: Enter input mode to create a new change set.
  - **`Enter`** (in input mode): Submit the name and create the change set.
  - **`Esc`** (in input mode): Cancel creation.
  - **`Backspace`** (in input mode): Delete last character.
- **`d`**: Delete the currently selected change set.
- **`f`**: Force apply the currently selected change set.
- **`k`**: Scroll log window up.
- **`j`**: Scroll log window down.
