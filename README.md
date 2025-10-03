# ☕ Grind: grind hard, code harder - Builds, without the headache

![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/anharhussainmiah/grind?style=for-the-badge&logo=github&label=Latest%20Release)
![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)
![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)

## 🚀 More Coffee, Less XML

Compared to modern build tools, Java build tools feel outdated and overly complex. With **grind** we're trying to bring a similar experiance as that of `npm` or `cargo`.

**grind** is a blazing-fast, Rust powered command line interface designed to take the friction out of **Maven**, **Gradle** and other Java Build tools. Tired of manually editing verbose `pom.xml` files or hunting for the right dependency version, or trying to get it actually build?

**grind** simplifies your project workflow by introducing the **`grind.yml`** manifest, providing a single, consistent source of truth for all your project configurations, Write less XML or configuration, manage more efficiently, and get back to writing code!

**TL;DR: grind the npm of Java**

## ✨ Main Features

| Feature                      | Description                                                                                | CLI Example                          |
| :--------------------------- | :----------------------------------------------------------------------------------------- | :----------------------------------- |
| **☕ Project Scaffolding**   | Quickly bootstrap a new Java project structure with a pre-configured `grind.yml` manifest. | `grind init my-new-service`          |
| **⚙️ Easy Jar Builds**       | Builds the entire project based on the `grind.yml` manifest into a Jar                     | `grind build`                        |
| **➕ Dependency Management** | Add and remove project dependencies directly from the command line                         | `grind add spring-boot mysql-driver` |
| **✅ Task Execution**        | The `grind.yml` can contain many custom tasks, similar to `package.json`                   | `grind task clean`                   |

## 📥 Installation

The **grind** CLI is distributed as a single, static binary built with Rust, meaning **no runtime dependencies** are needed—just download and run!

We recommend downloading the appropriate binary for your system from the [GitHub Releases page](https://github.com/anharhussainmiah/grind/releases/latest).

### Linux / macOS

1.  **Download the latest binary** (replace `X.Y.Z` with the actual version and adjust the file name for your OS/architecture):

    ```bash
    # For Linux (x86_64)
    wget -O grind [https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/cfe-x86_64-unknown-linux-gnu](https://github.com/YOUR_GITHUB_USERNAME/cfe/releases/download/vX.Y.Z/cfe-x86_64-unknown-linux-gnu)

    # OR for macOS (Apple Silicon/M-series)
    wget -O grind [https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/cfe-aarch64-apple-darwin](https://github.com/YOUR_GITHUB_USERNAME/cfe/releases/download/vX.Y.Z/cfe-aarch64-apple-darwin)
    ```

2.  **Make it executable**:
    ```bash
    chmod +x grind
    ```
3.  **Move it to your PATH** (e.g., `/usr/local/bin`):
    ```bash
    sudo mv grind /usr/local/bin/
    ```

## Current Progress/Road Map

- [x] Scaffold New Project
- [x] Install all dependencies
- [ ] Compile and build Jar file
- [ ] Compile and run Project
- [ ] Run a specific task as define in the `grind.yml` manifest
- [ ] List all available custom tasks
- [ ] Add a dependency
- [ ] Remove a dependency

### Longer Term goals

- [ ] Test Runner
- [ ] Manage Java SDK versions a bit like "Node Version Manager" or "rustup"
- [ ] A Java formatter e.g a bit like "cargo fmt"

## 💡 Basic Usage Examples

Once installed, managing your Maven projects is only a single command away.

```shell
                     /$$                 /$$
                    |__/                | $$
  /$$$$$$   /$$$$$$  /$$ /$$$$$$$   /$$$$$$$
 /$$__  $$ /$$__  $$| $$| $$__  $$ /$$__  $$
| $$  \ $$| $$  \__/| $$| $$  \ $$| $$  | $$
| $$  | $$| $$      | $$| $$  | $$| $$  | $$
|  $$$$$$$| $$      | $$| $$  | $$|  $$$$$$$
 \____  $$|__/      |__/|__/  |__/ \_______/
 /$$  \ $$
|  $$$$$$/
 \______/

        Grind hard, code harder v0.0.1
          - "builds, without the headache"


Usage: grind <COMMAND>

Commands:
  init     Initializes a new grind project structure
  install  Install all the external libraries as defined in the grind.yml dependencies
  build    Builds the project using the configuration in grind.yml
  run      Runs the project (e.g., mvn spring-boot:run)
  add      Adds a dependency to the project's grind.yml
  remove   Removes a dependency from the project's grind.yml
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

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

### 2. Compile, build, and create a Jar file

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
grind run
```

## No FAT Jars!

I was deciding on if `grind` should create "fat Jars", but then I realised that in modern development we end up creating docker containers, and creating a fat jar doesn't make all that sense, AND you potentially lose out on caching!

Instead of creating a fat jar, just copy the `lib` folder, you end up with a single image anyway, but with the advantage of proper caching meaning your images will only update the actual application which could be kilobytes vs hundreds of megabytes!

## No Windows Support.

Techinally `grind` could support Windows, but I don't have the engery to make that happen, it's open source and I would massively welcome contributions, if Windows is important to you!
