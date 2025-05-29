# Tetris HTML

This project is an implementation of the beloved puzzle game, Tetris.

## What is Tetris?

Tetris is a tile-matching puzzle game where players aim to create complete horizontal lines of blocks by manipulating falling tetrominoes (shapes composed of four square blocks each). When a line is completed, it disappears, and blocks above fall to fill the space. The game ends when the playing field fills up and new tetrominoes can no longer enter.

## About This Project

This version of Tetris is built with Rust, utilizing the Leptos framework for its reactive web interface and Tauri to package it as a desktop application.

It aims to deliver the classic Tetris gameplay experience.

### Key Features (Planned/Implemented)

The game includes or plans to include the following features:

*   Classic Tetris gameplay: Manipulate falling tetrominoes to form complete lines.
*   Score tracking.
*   Preview of the next upcoming piece.
*   Increasing difficulty as the game progresses.

## Design

This section outlines the architecture and core technologies used in the Tetris HTML project.

### Architecture

The application follows a modern web-centric desktop application model:

*   **Web-based Frontend:** The core game logic and user interface are built as a web application. This is where the interactive Tetris gameplay happens.
    *   The frontend is developed using the **Leptos framework**, a full-stack Rust framework that compiles to WebAssembly. This allows for writing reactive UI components and game logic entirely in Rust.
*   **Desktop Wrapper:** To provide a native desktop experience (e.g., an executable file, window management), the web-based frontend is wrapped using **Tauri**.
    *   Tauri packages the Leptos-generated WebAssembly and associated web assets into a lightweight, cross-platform desktop application. It essentially embeds a web view to render the game interface.

This architecture allows for a single Rust codebase for the core game and UI logic, which can be run in a web browser during development and then packaged for desktop distribution.

### Technologies Used

The following core technologies power this Tetris game:

*   **Rust:** The primary programming language used for both the frontend game logic (via Leptos) and the desktop application layer (via Tauri). Its performance and safety features make it suitable for game development.
*   **Leptos:** A modern, full-stack Rust web framework used to build the user interface and manage the game's state. It enables reactive UI development with Rust, compiling to WebAssembly.
*   **WebAssembly (WASM):** The compilation target for the Leptos frontend. WASM allows the Rust code to run efficiently in web browsers and, consequently, within Tauri's webview component.
*   **Tauri:** A toolkit for building lightweight, secure, and cross-platform desktop applications using web technologies. It wraps the Leptos web application, providing native functionalities and a distributable binary.
*   **HTML/CSS/JavaScript:** While the primary development is in Rust, these foundational web technologies are inherently used. Leptos generates the necessary HTML structure and can interface with JavaScript, while CSS is used for styling the game's appearance. Tauri uses a system webview, which renders these technologies.

## Usage Instructions

This section guides you through setting up the project for development and how to play the game.

### Prerequisites

Before you begin, ensure you have the following tools installed:

*   **Rust (with cargo):** The core programming language and build system. If you don't have it, install it via [rustup](https://rustup.rs/).
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
*   **Trunk:** A WASM web application bundler for Rust. Install it using cargo:
    ```bash
    cargo install trunk
    ```
*   **Tauri CLI:** The command-line interface for Tauri. Install it using cargo:
    ```bash
    cargo install tauri-cli
    ```
    *Note: Depending on your system and Tauri version, you might also need Node.js for some Tauri operations, though often the Rust-based CLI is sufficient for development.*
*   **Web Browser:** A modern web browser (e.g., Firefox, Chrome) is required for the `trunk serve` development mode.

### Development Setup

To run the project in development mode:

1.  **Clone the repository:**
    If you haven't already, clone the project repository to your local machine.
    ```bash
    # Example: git clone https://github.com/username/repository-name.git
    ```
    (Replace the URL with the actual repository URL)
2.  **Navigate to the project directory:**
    ```bash
    cd your-project-directory-name 
    ```
    (Replace `your-project-directory-name` with the actual folder name)
3.  **Run the frontend development server:**
    Open a terminal and run:
    ```bash
    trunk serve
    ```
    This will build the Leptos frontend and serve it, usually on `http://127.0.0.1:8080`. The game can be played directly in the browser at this stage. This command is also specified as `beforeDevCommand` in the `tauri.conf.json` file, meaning Tauri can run it automatically.

4.  **Run the Tauri development application:**
    In another terminal (or if `trunk serve` is handled automatically by the next step), run:
    ```bash
    cargo tauri dev
    ```
    This command will build and launch the Tetris game in a native desktop window. It will also automatically run `trunk serve` if not already running and configured as the `beforeDevCommand`.

### How to Play

The objective of Tetris is to score points by clearing horizontal lines of blocks. Manipulate the falling tetrominoes (shapes made of four blocks) to create solid horizontal lines. When a line is complete, it disappears, and any blocks above will fall to fill the space. The game ends if the blocks stack up to the top of the playing field.

**Controls:**

*   **Left Arrow Key:** Move the falling block left.
*   **Right Arrow Key:** Move the falling block right.
*   **Down Arrow Key:** Speed up the block's descent (soft drop, one step).
*   **Up Arrow Key:** Rotate the block.
*   **Spacebar:** Drop the block instantly to the bottom (hard drop).
*   **P Key:** Pause or resume the game.

## Deployment Instructions

This section explains how to build the Tetris application for production/distribution.

### Building for Production

To build the application for production, run the following command in the project's root directory:

```bash
cargo tauri build
```

This command orchestrates the entire build process:

1.  **Frontend Build:** It first executes the `beforeBuildCommand` specified in the `tauri.conf.json` file. For this project, this is `trunk build --release`. This command compiles the Leptos frontend application (WebAssembly and JavaScript/CSS assets) in release mode and places the output in a `dist` directory (or similar, as configured by Trunk).
2.  **Tauri Application Build:** After the frontend is built, Tauri compiles the Rust application and bundles the frontend assets into a distributable format for the target platform(s).

The built artifacts (executables, installers, etc.) can typically be found in the `src-tauri/target/release/bundle/` directory. The exact subdirectories and file types will vary depending on the operating system and the specific targets being built (e.g., `.msi` for Windows, `.dmg` or `.app` for macOS, `.deb` or `.AppImage` for Linux).

### Supported Platforms

Based on the project configuration (e.g., `"targets": "all"` in the `tauri.conf.json` file), the application is intended to be buildable for the following platforms:

*   **Windows**
*   **macOS** (Intel and Apple Silicon)
*   **Linux**

**Note on Cross-Compilation:** While Tauri supports building for all these platforms, actually producing a build for a specific platform typically requires running the `cargo tauri build` command on that platform. For example, to build the macOS `.dmg` or `.app` bundle, you would usually run the build command on a macOS machine. Cross-compilation is possible for some targets but may require additional setup of specific toolchains and SDKs.

This project serves as a demonstration of building a web-based game with Rust, Leptos, and Tauri, showcasing how these technologies can be combined to create engaging applications.

We hope you enjoy playing!

---

## Tetris Gymnasium Environment for Reinforcement Learning

This project also includes a [Gymnasium](https://gymnasium.farama.org/) environment, allowing you to train reinforcement learning (RL) agents to play Tetris. The core Tetris game logic is implemented in Rust for performance, and the Python environment interfaces with this compiled Rust core.

### Overview

The `TetrisEnv` class, found in `tetris_env.py`, provides a standard Gymnasium API for interacting with the Tetris game. This allows researchers and developers to easily apply various RL algorithms to learn Tetris strategies.

### Prerequisites

To use the Tetris Gymnasium environment, you'll need:

*   **Rust Toolchain:**
    *   Install Rust and Cargo from [https://rustup.rs/](https://rustup.rs/).
    *   Ensure `cargo` is in your system's PATH.
*   **Python:**
    *   Python 3.8 or newer is recommended.
    *   `pip` (Python's package installer) is required.

### Building the Rust Core

The Rust core of the Tetris game needs to be compiled into a dynamic library (e.g., `.so` on Linux, `.dll` on Windows, `.dylib` on macOS) so that the Python environment can call it.

1.  **Navigate to the project root directory** (where `Cargo.toml` is located).
2.  **Build the library using Cargo:**
    *   For a debug build (faster compilation, not optimized):
        ```bash
        cargo build
        ```
    *   For a release build (slower compilation, optimized for performance):
        ```bash
        cargo build --release
        ```
3.  **Locate the compiled library:**
    *   The library will be named `libtetris_core.so` (Linux), `tetris_core.dll` (Windows), or `libtetris_core.dylib` (macOS).
    *   It will be located in:
        *   `target/debug/` for debug builds.
        *   `target/release/` for release builds.

    The `tetris_env.py` script attempts to find this library automatically, assuming it's in `target/debug/` relative to the project root.

### Setting up the Python Environment

1.  **Create and activate a virtual environment (recommended):**
    Open your terminal in the project root directory and run:
    ```bash
    python -m venv .venv
    # On Linux/macOS:
    source .venv/bin/activate
    # On Windows (Command Prompt):
    # .venv\Scripts\activate.bat
    # On Windows (PowerShell):
    # .venv\Scripts\Activate.ps1
    ```
2.  **Install Python dependencies:**
    With your virtual environment activated, install the required packages:
    ```bash
    pip install gymnasium numpy
    ```

### Running the Environment (Python Example)

The following Python script demonstrates how to import and use the `TetrisEnv`. Ensure that the compiled Rust library is discoverable (e.g., by being in `target/debug/` or `target/release/`, or by providing the `lib_path` argument to `TetrisEnv`).

```python
from tetris_env import TetrisEnv
import gymnasium as gym # Ensure gymnasium is imported if not within TetrisEnv's example

# The TetrisEnv will attempt to find the compiled library in standard locations
# (e.g., ./target/debug/libtetris_core.so).
# If your library is elsewhere, provide the path:
# env = TetrisEnv(lib_path="path/to/your/target/debug_or_release/library_file")
env = TetrisEnv() 

obs, info = env.reset()
terminated = False
total_reward = 0

print(f"Starting Tetris RL Environment. Observation shape: {obs.shape}")

for step in range(1000): # Run for a maximum of 1000 steps
    action = env.action_space.sample()  # Replace with your agent's action
    
    obs, reward, terminated, truncated, info = env.step(action)
    total_reward += reward
    
    # Optional: Render the environment
    # env.render() 
    # import time; time.sleep(0.05) # Slow down for human viewing if rendering

    if (step + 1) % 100 == 0:
        print(f"Step: {step + 1}, Score: {info.get('score', 0)}, Total Reward: {total_reward:.2f}")

    if terminated or truncated:
        print(f"Episode finished after {step + 1} steps.")
        print(f"Final Score: {info.get('score', 0)}, Total Reward for episode: {total_reward:.2f}")
        
        # Reset for the next episode
        obs, info = env.reset()
        total_reward = 0 
        # if you want to stop after one episode, add a break here
        # break 

env.close()
print("Environment closed.")
```

### Running Tests

Unit tests for the Python environment are located in `test_tetris_env.py`. To run them:

1.  Ensure the Rust library is built (e.g., `cargo build`).
2.  Ensure Python dependencies are installed.
3.  Run the tests from the project root directory:
    ```bash
    python -m unittest test_tetris_env.py
    ```
