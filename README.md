# ☕ Grind: Builds, without the headache

![Grind](logo.png)

![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/anharhussainmiah/grind?style=for-the-badge&logo=github&label=Latest%20Release)
![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)
![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)

## 🚀 More Coffee, Less XML and configuration

Compared to modern build tools such as **cargo** and **npm** Java build tools feel outdated and overly complex, and they seem to be relegated to the background only to be used by the IDE or CI/CD tooling.

**grind** is a hassle free, Rust powered CLI designed to remove the friction and improve the DX compared to the current set of Java build tools such as **Maven** and **Gradle**. Are you tired of fighting and wasting time with these complex build tools? Then maybe it's time to try a new tool!

**grind** simplifies your project workflow by introducing the **`grind.yml`** manifest, providing a single consistent source of truth for all your projects. Write less XML or build configurations, manage builds more efficiently, and get back to writing code!

**TL;DR: grind the npm of Java**

## ✨ Main Features

| Feature                        | Description                                                                      | CLI Example                           |
| :----------------------------- | :------------------------------------------------------------------------------- | :------------------------------------ |
| **☕ Project Scaffolding**     | Quickly bootstrap a new Java project with a pre-configured `grind.yml` manifest. | `grind new com.example/HelloWorld`    |
| **📥 Easy Dependency Install** | Install all your projects dependencies                                           | `grind install`                       |
| **▶️ Quickly Run Project**     | Compile your code and run it                                                     | `grind run`                           |
| **⚙️ Production Jar Builds**   | Compile and build your production Jar                                            | `grind build`                         |
| **➕ Dependency Management**   | Add and remove project dependencies directly from the command line               | `grind add org.postgresql/postgresql` |
| **✅ Task Execution**          | Define and run custom tasks much like `package.json`                             | `grind task clean`                    |

## 📥 Installation

The **grind** CLI is distributed as a single, static binary built with Rust, meaning **no runtime dependencies** are needed — just download and run!

We recommend downloading the appropriate binary for your system from the [GitHub Releases page](https://github.com/anharhussainmiah/grind/releases/latest).

### Linux / macOS

1.  **Download the latest binary** (replace `X.Y.Z` with the actual version and adjust the file name for your OS/architecture):

    ```bash
    # For Linux (x86_64)
    wget -O grind [https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/grind-x86_64-unknown-linux-musl](https://github.com/anharhussainmiah/grind/releases/download/vX.Y.Z/grind-x86_64-unknown-linux-musl)

    # OR for macOS (Apple Silicon/M-series)
    wget -O grind [https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/grind-aarch64-apple-darwin](https://github.com/anharhussainmiah/grind/releases/download/vX.Y.Z/grind-aarch64-apple-darwin)
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

- [x] ✅ Scaffold New Project
- [x] ⏳ Install all dependencies
  - [x] ✅ Use "newest" strategy for artifact collisions
  - [x] ✅ Generate "grind.lock" file, re-generate on add/remove or mismatch on "install"
  - [ ] ⚠️ Correctly handle super POM via `<parent>`, `BOM` imports, and `<dependencyManagement>` resolution
  - [ ] ⚠️ Handle exclusions
- [x] ✅ Compile and build Jar file
- [x] ✅ Compile and run Project
- [x] ✅ Run a specific task as define in the `grind.yml` manifest
- [x] ✅ List all available custom tasks
- [x] ✅ Add a dependency
- [x] ✅ Remove a dependency

## Long Term Goals

- [ ] Test Runner
- [ ] Manage Java SDK versions a bit like "Node Version Manager" or "rustup"
- [ ] Other Repositories than Maven or Support custom/private repos?
- [ ] Multiple Project Scaffolds
- [ ] A Java formatter e.g a bit like "cargo fmt"

## 💡 Basic Usage Examples

Once installed, managing your Java projects is only a single command away.

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

### 1. Create a New Project

To create a new project, type the project name using the format `<groupId>/<artifactId>`, this creates a new directory and project stucture, and the `grind.yml` manifest file.

```bash
# Creates a new directory named 'PaymentsApi' and initialises files inside it
grind new com.example/PaymentsApi
cd PaymentsApi
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

### 2. Install all dependencies

To install all your projects dependencies that are defined in the `grind.yml` file, simple invoke the following:

```bash
grind install
```

### 3. Add Dependencies

To add one or more dependency to your project invoke the `add` sub command, make sure to use the format `<groupId>/<artifactId>`:

```bash
grind add org.postgresql/posgresql
# you can even pin a specific version
grind add org.postgresql/posgresql@42.7.7
# you can add multiple dependecies at the same time, just separate by space
grind add org.jsoup/jsoup org.apache.commons/commons-csv
```

### 4. Remove Dependecnies

Remove dependencies just as easily, use the format `<groupId>/<artifactId>`

```bash
grind remove org.postgresql/posgresql
```

### 5. Run the Project

To compile and run your project, simpley invoke the following:

```bash
grind run
```

### 6. Compile and Package up a final Jar executable

To build your production `jar` simply invoke the following:

```bash
grind build
```

Your compiled and package `jar` will be available in the `build/` folder, however keep in mind because this is not
a "fat jar" or "uber jar", your dependecies which are located in the `libs/` folder would need to be relative to where the `jar` is.

So for production you would need to include both your `jar` file as well as the `libs/` folder.

### 7. Optional run custom tasks

Much like the tasks that can be set in the `package.json` grind also has a similar feature, you can list the current available task as follows:

```shell
grind task list
available tasks:

 - info
 - clean

```

to create a task, simply edit your `grind.yml` e.g:

```yaml
tasks:
  clean: "rm -rf target/"
  copy-jar: "cp build/*.jar ."
```

then to run your custom task just do:

```shell
grind task copy-jar
```

### Dependencies

Grind assumes the following are already installed on your machine:

- Bash
- Java

## No FAT Jars!

I was deciding on if `grind` should be able to create "fat Jars", but then I realised that in modern development we end up creating container images. In that context creating a fat jar doesn't make all that sense! AND you potentially lose out on caching!

Instead of creating a fat jar, just copy the `libs` folder, you'll end up with a single container image anyway, but with the advantage of proper caching layers meaning, next time your update your images, it will only update the actual application layer which could be just a few kilobytes vs hundreds of megabytes!

## No Windows Support.

Techinally `grind` could be made to support Windows _(swithing out bash for poweshell, and minor changes in classpaths)_, but I don't have the energy to make that happen! It's open source and I would massively welcome all contributions for any feature(s) that you would really like to see.

## Coding Design

I'm following the `YAGNI` and `KISS` as well as _"Deliver the simplest solution that works."_ approach to software design. I also do not like lots of nested folders, so I try to organise the codes into their own
modules, the fewer the indirection the easier it is to reason about the code.

## Contributions

I'm open to contributions, suggestions, and ideas, etc but please be kind!

If you've found this project helpful, please leave a 🌟 or help spread the word!
