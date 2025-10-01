# ☕ Grind: grind hard, code harder - Builds, without the headache

![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/anharhussainmiah/grind?style=for-the-badge&logo=github&label=Latest%20Release)
![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)
![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)

## 🚀 More Coffee, Less XML

Compared to modern build tools, Java build tools feel outdated and overly complex. With **grind** we're trying to bring a similar experiance as very popular tools like `npm` and `cargo`.

**grind** is a blazing-fast, Rust powered command line interface designed to take the friction out of **Maven** and other Java Build tools. Tired of manually editing verbose `pom.xml` files or hunting for the right dependency version?

**grind** simplifies your project workflow by introducing the **`grind.yml`** manifest, providing a single, consistent source of truth for all your project configurations, Write less XML, manage more efficiently, and get back to writing code.

## ✨ Main Features

| Feature                      | Description                                                                                                     | CLI Example                          |
| :--------------------------- | :-------------------------------------------------------------------------------------------------------------- | :----------------------------------- |
| **☕ Project Scaffolding**   | Quickly bootstrap a new Java project structure with a pre-configured `grind.yml` manifest.                      | `grind init my-new-service`          |
| **⚙️ Config-Driven Builds**  | Builds the entire project based on the `grind.yml` manifest                                                     | `grind build`                        |
| **➕ Dependency Management** | Add and remove project dependencies directly from the command line without ever opening a `pom.xml` file again. | `grind add spring-boot mysql-driver` |
| **✅ Task Execution**        | The `grind.yml` can contain many custom tasks, similar to `package.json`                                        | `grind task web`                     |

## 📥 Installation

The **grind** CLI is distributed as a single, static binary built with Rust, meaning **no runtime dependencies** are needed—just download and run!

We recommend downloading the appropriate binary for your system from the [GitHub Releases page](https://github.com/anharhussainmiah/grind/releases/latest).

### Linux / macOS

1.  **Download the latest binary** (replace `X.Y.Z` with the actual version and adjust the file name for your OS/architecture):

    ```bash
    # For Linux (x86_64)
    wget -O cfe [https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/cfe-x86_64-unknown-linux-gnu](https://github.com/YOUR_GITHUB_USERNAME/cfe/releases/download/vX.Y.Z/cfe-x86_64-unknown-linux-gnu)

    # OR for macOS (Apple Silicon/M-series)
    wget -O cfe [https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/cfe-aarch64-apple-darwin](https://github.com/YOUR_GITHUB_USERNAME/cfe/releases/download/vX.Y.Z/cfe-aarch64-apple-darwin)
    ```

2.  **Make it executable**:
    ```bash
    chmod +x grind
    ```
3.  **Move it to your PATH** (e.g., `/usr/local/bin`):
    ```bash
    sudo mv grind /usr/local/bin/
    ```

### Windows (PowerShell)

1.  Download the `grind-x86_64-pc-windows-msvc.exe` file from the [Releases page](https://github.com/anharhussainmiah/grind/releases/latest).
2.  Rename the downloaded file to `cfe.exe`.
3.  Place `grind.exe` in a directory included in your system's `Path` environment variable (e.g., `C:\Users\YourName\.cargo\bin`).

---

## 💡 Basic Usage Examples

Once installed, managing your Maven projects is only a single command away.

### 1. Initialize a New Project

Creates a new directory and project stucture, and the essential `grind.yml` manifest file.

```bash
# Creates a new directory named 'payments-api' and initializes files inside it
grind init payments-api
cd payments-api
```

## Output

```YAML
project:
  groupId: "com.example"
  artifactId: "payments-api"
  version: "1.0.0"
  name: "payments-api"
  description: ""

  dependencies:
    - groupId: "junit"
      artifactId: "junit"
      version: "4.13.2"
      scope: "test"

  tasks:
    clean: "rm -rf target/"
```

### 2. Build the Project

Runs the build process

```bash
grind build
```

### 3. Add Dependencies

Specify one or more dependencies to automatically insert them into your `pom.xml`.

```bash
grind add spring-boot-starter-web postgresql-driver
```

### 4. Remove Dependecnies

Remove dependencies just as easily.

```bash
grind remove postgresql-driver
```

### 5. Run the Project

Executes the project using the configured settings.

```bash
grind task run
```

# 🤖 AI Assisted Code Disclaimer

The source code for **cfe** was partially generated, reviewed, and enhanced using Google's generative AI models. While this assistance significantly accelerated development, the code has been thoroughly reviewed, tested, and audited for security, performance, and correctness by human developers. We are committed to maintaining a high standard of quality for this project.
