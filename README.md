# ☕ Grind: Builds, without the headache

![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/anharhussainmiah/grind?style=for-the-badge&logo=github&label=Latest%20Release)
![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)
![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)

## 🚀 More Coffee, Less XML and configuration

Compared to modern build tools such as **cargo** and **npm** Java build tools feel outdated and overly complex, and they seem to be relegated to the background only to be interfaced by the IDE or CI/CD tooling.

**grind** is a blazing-fast, Rust powered command line interface designed to take the friction and UX out of current Java build tools such as **Maven** and **Gradle**. Are you tired of fighting and wasting time with these complex build tools?

**grind** simplifies your project workflow by introducing the **`grind.yml`** manifest, providing a single consistent source of truth for all your project. Write less XML or build configurations, manage more efficiently, and get back to writing code!

**TL;DR: grind the npm of Java**

## ✨ Main Features

| Feature                      | Description                                                                                | CLI Example                           |
| :--------------------------- | :----------------------------------------------------------------------------------------- | :------------------------------------ |
| **☕ Project Scaffolding**   | Quickly bootstrap a new Java project structure with a pre-configured `grind.yml` manifest. | `grind new com.example/HelloWorld`    |
| **⚙️ Easy Jar Builds**       | Compile and build your production Jar                                                      | `grind build`                         |
| **➕ Dependency Management** | Add and remove project dependencies directly from the command line                         | `grind add org.postgresql/postgresql` |
| **✅ Task Execution**        | The `grind.yml` can contain many custom tasks, similar to `package.json`                   | `grind task clean`                    |

## 📥 Installation

The **grind** CLI is distributed as a single, static binary built with Rust, meaning **no runtime dependencies** are needed — just download and run!

We recommend downloading the appropriate binary for your system from the [GitHub Releases page](https://github.com/anharhussainmiah/grind/releases/latest).

### Linux / macOS

1.  **Download the latest binary** (replace `X.Y.Z` with the actual version and adjust the file name for your OS/architecture):

    ```bash
    # For Linux (x86_64)
    wget -O grind [https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/cfe-x86_64-unknown-linux-gnu](https://github.com/YOUR_GITHUB_USERNAME/grind/releases/download/vX.Y.Z/cfe-x86_64-unknown-linux-gnu)

    # OR for macOS (Apple Silicon/M-series)
    wget -O grind [https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/cfe-aarch64-apple-darwin](https://github.com/YOUR_GITHUB_USERNAME/grind/releases/download/vX.Y.Z/cfe-aarch64-apple-darwin)
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
- [x] Compile and build Jar file
- [x] Compile and run Project
- [x] Run a specific task as define in the `grind.yml` manifest
- [x] List all available custom tasks
- [x] Add a dependency
- [x] Remove a dependency

### Longer Term goals

- [ ] Test Runner
- [ ] Manage Java SDK versions a bit like "Node Version Manager" or "rustup"
- [ ] Other Repositories than Maven or Support custom/private repos?
- [ ] Multiple Project Scaffolds
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

        - "builds, without the headache"
                  v0.2.0


Usage: grind <COMMAND>

Commands:
  new      Scaffolds a new Java project with a grind.yml file
  install  Download all the external libraries and dependencies as defined in the grind.yml
  build    Compile the project and builds a jar file
  run      Compile and run the project
  add      Adds a dependency to the project's grind.yml
  remove   Removes a dependency from the project's grind.yml
  task     Run a custom task as defiend in the grind.yml, e.g grind task clean
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### 1. Initialize a New Project

Creates a new directory and project stucture, and the essential `grind.yml` manifest file. Type the new project using the format `<groupId>/<artifactId>`

```bash
# Creates a new directory named 'payments-api' and initializes files inside it
grind new com.example/PaymentsApi
cd payments-api
```

## Output

```YAML
project:
  groupId: "com.example"
  artifactId: "PaymentsApi"
  version: "1.0.0"
  name: "PaymentsApi"
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

Specify one or more dependencies to automatically insert them into your `grind.yml`, use the format `<groupId>/<artifactId>`

```bash
grind add org.postgresql/posgresql
```

### 4. Remove Dependecnies

Remove dependencies just as easily, use the format `<groupId>/<artifactId>`

```bash
grind remove org.postgresql/posgresql
```

### 5. Run the Project

Executes the project using the configured settings.

```bash
grind run
```

### Dependencies

Grind assumes the following are already installed on your machine:

- Bash
- Java

## No FAT Jars!

I was deciding on if `grind` should be able to create "fat Jars", but then I realised that in modern development we end up creating containers images. Creating a fat jar doesn't make all that sense, AND you potentially lose out on caching!

Instead of creating a fat jar, just copy the `lib` folder, you end up with a single image anyway, but with the advantage of proper caching meaning next time your update your images, it will only update the actual application layer which could be kilobytes vs hundreds of megabytes!

## No Windows Support.

Techinally `grind` could be made to support Windows, but I don't have the energy to make that happen! It's open source and I would massively welcome all contributions for any feature(s) that you would really like to see

## Coding Design

I following the `YAGNI` and `KISS` as well as "Doing the minimal, to make it work"
