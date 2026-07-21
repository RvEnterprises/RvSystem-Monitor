# 🚀 RvSystem Monitor

[![Android](https://img.shields.io/badge/Platform-Android-3DDC84?logo=android&logoColor=white)](https://developer.android.com)
[![Kotlin](https://img.shields.io/badge/Language-Kotlin-7F52FF?logo=kotlin&logoColor=white)](https://kotlinlang.org)
[![Rust](https://img.shields.io/badge/Backend-Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Latest Release](https://img.shields.io/github/v/release/Rve27/RvSystem-Monitor)](https://github.com/Rve27/RvSystem-Monitor/releases)
[![License](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](LICENSE)
[![Compose](https://img.shields.io/badge/UI-Jetpack%20Compose-4285F4?logo=jetpackcompose&logoColor=white)](https://developer.android.com/compose)
[![Downloads](https://img.shields.io/github/downloads/Rve27/RvSystem-Monitor/total?logo=github&color=FF69B4)](https://github.com/Rve27/RvSystem-Monitor/releases)

**RvSystem Monitor** is a high-performance system monitoring solution for Android, merging the expressive power of **Jetpack Compose** with the raw efficiency of **Rust**. It provides low-level hardware insights while maintaining a modern, buttery-smooth user experience.

---

## 📖 Table of Contents
- [🚀 Overview](#-overview)
- [📸 Screenshots](#-screenshots)
- [🛠️ Tech Stack](#-tech-stack)
- [📂 Project Structure](#-project-structure)
- [🏗️ Architecture](#-architecture)
- [⚙️ Getting Started](#-getting-started)
- [🤝 Contributing](#-contributing)
- [💬 Support](#-support)
- [📜 License](#-license)

---

## 🚀 Overview
RvSystem Monitor bridges the gap between high-level UI frameworks and low-level system APIs. By utilizing a Rust-based backend, it minimizes the performance overhead typically associated with frequent polling of kernel files like `/proc` and `/sys`. This hybrid approach allows for real-time monitoring of CPU frequencies, GPU drivers, battery health, and memory usage without compromising the device's responsiveness.

Built with **Material 3 Expressive**, the application offers a visually rich experience with adaptive layouts and sophisticated transitions, making system diagnostics both powerful and beautiful.

---

## 📸 Screenshots

<p align="center">
  <img src="fastlane/metadata/android/en-US/images/phoneScreenshots/screenshot_1.jpg" width="32%" />
  <img src="fastlane/metadata/android/en-US/images/phoneScreenshots/screenshot_2.jpg" width="32%" />
  <img src="fastlane/metadata/android/en-US/images/phoneScreenshots/screenshot_3.jpg" width="32%" />
</p>
<p align="center">
  <img src="fastlane/metadata/android/en-US/images/phoneScreenshots/screenshot_4.jpg" width="32%" />
  <img src="fastlane/metadata/android/en-US/images/phoneScreenshots/screenshot_5.jpg" width="32%" />
  <img src="fastlane/metadata/android/en-US/images/phoneScreenshots/screenshot_6.jpg" width="32%" />
</p>

---


## 🛠️ Tech Stack

- **Languages**: Kotlin 2.4.0, Rust (Edition 2024), C (via `libc`).
- **Frameworks**: Jetpack Compose (BOM 2026.06.00), Material 3 Expressive, Hilt DI.
- **Native Bridge**: JNI via `jni-rs`, `cargo-ndk`.
- **Infrastructure**: Gradle Kotlin DSL, Android NDK 30.0, Fastlane.
- **Distribution**: Multiple flavors (GitHub, F-Droid) with toggleable update mechanisms.
- **Libraries**: Retrofit 3.0, OkHttp 5.4, Coil 3.5, Jetpack DataStore.

---

## 📂 Project Structure
```text
RvSystem-Monitor/
├── app/                  # Android application module (Kotlin)
│   ├── src/main/java/    # UI, ViewModels, and JNI bridge declarations
│   ├── src/main/jniLibs/ # Compiled native shared libraries (.so)
│   ├── src/main/res/     # App resources and icons
│   └── build.gradle.kts  # Gradle configuration for Android
├── rust/                 # Native monitoring backend (Rust)
│   ├── src/              # Kernel parsing, JNI implementation, and drivers
│   ├── Cargo.toml        # Rust package and dependency metadata
│   └── README.md         # Documentation for the Rust sub-system
├── gradle/               # Build system scripts and version catalogs
├── fastlane/             # Automation for screenshots and deployments
├── LICENSE               # GNU GPL v3.0
└── README.md             # This file
```

---

## 🏗️ Architecture

The project adheres to **Clean Architecture** principles, ensuring a strict separation of concerns and high maintainability.

### The Hybrid Core
- **UI Orchestration**: The Kotlin layer manages the application lifecycle and UI state. It uses ViewModels to expose reactive data streams from Hilt-injected repositories.
- **Native Data Source**: The Rust layer handles the "heavy lifting". It parses system files and interacts with hardware drivers. By mirroring the Linux kernel's structure (`kernel/` for CPU, `mm/` for Memory, and `drivers/` for GPU), it provides an idiomatic and high-performance data source.
- **Optimized JNI Bridge**: Instead of frequent fine-grained calls, the bridge is designed for **batch data retrieval**. Single calls fetch complete data sets (e.g., all CPU metrics at once), significantly reducing context-switching overhead between the JVM and Native code.

---

## ⚙️ Getting Started

### Prerequisites
- **Android Studio** (Ladybug 2024.2.1 or newer)
- **Rust Toolchain** ([rustup.rs](https://rustup.rs/)): Edition 2024 (Stable 1.85+ recommended).
  - Add Android targets: `rustup target add aarch64-linux-android armv7-linux-androideabi`
- **Android NDK** (Version `30.0.14904198` configured)
- **cargo-ndk**: `cargo install cargo-ndk`

### Installation & Build
1. **Clone the repository**:
   ```bash
   git clone https://github.com/Rve27/RvSystem-Monitor.git
   cd RvSystem-Monitor
   ```
2. **Build Native Libraries**:
   ```bash
   ./gradlew :app:buildRustLibraries
   ```
3. **Build the application**:
   Choose the variant you want to build:
   - **GitHub Variant** (Includes auto-updater):
     ```bash
     ./gradlew assembleGithubRelease
     ```
   - **F-Droid Variant** (No updater):
     ```bash
     ./gradlew assembleFdroidRelease
     ```
4. **Install and run (Debug)**:
   Connect an Android device (API 34+) and run:
   ```bash
   ./gradlew installGithubDebug
   ```

---

## 🤝 Contributing
We welcome contributions from the community! Whether you are fixing a bug, adding a feature, or improving documentation, please read our [CONTRIBUTING.md](CONTRIBUTING.md) to get started.

## 💬 Support
- **Issues**: [GitHub Issues](https://github.com/Rve27/RvSystem-Monitor/issues) for bug reports and feature requests.
- **Discussions**: [GitHub Discussions](https://github.com/Rve27/RvSystem-Monitor/discussions) for questions and ideas.

## 📜 License
This project is licensed under the **GNU General Public License v3.0**. See the [LICENSE](LICENSE) file for details.

---
<p align="center">
  Built with ❤️ for the Android Community.
</p>
