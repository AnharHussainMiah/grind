# ☕ Grind: Java Builds, without the headache

![Grind](logo.png)

![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/anharhussainmiah/grind?style=for-the-badge&logo=github&label=Latest%20Release)
![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)
![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)

## 🚀 More Coffee, Less XML and configuration

Compared to modern build tools such as **cargo** and **npm** Java build tools feel outdated and overly complex, and they seem to be relegated to the background only to be used by the IDE or CI/CD tooling.

**grind** is a hassle free, Rust powered CLI designed to remove the friction and improve the DX compared to the current set of Java build tools such as **Maven** and **Gradle**. Are you tired of fighting and wasting time with these complex build tools and really heavy IDEs? Then maybe it's time to try a new alternative! `grind` is perfect for those who prefer "batteries included" CLIs toolchains and lightweight editors such as Visual Studio Code, VIM/Emacs etc.

**grind** simplifies your project workflow by introducing the **`grind.yml`** manifest, providing a single consistent source of truth for all your projects. Write less XML or build configurations, manage builds more efficiently, and get back to writing code!

**TL;DR: grind the cargo of Java**

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
    wget -O grind https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/grind-x86_64-unknown-linux-musl

    # OR for macOS
    wget -O grind https://github.com/anharhussainamiah/grind/releases/download/vX.Y.Z/grind-x86_64-apple-darwin
    ```

2.  **Make it executable**:
    ```bash
    chmod +x grind
    ```
3.  **Move it to your PATH** (e.g., `/usr/local/bin`):
    ```bash
    sudo mv grind /usr/local/bin/
    ```

## Current Progress/Roadmap

- [x] ✅ Scaffold New Project
- [x] ✅ Install all dependencies
  - [x] ✅ Use "newest" strategy for artifact collisions
  - [x] ✅ Generate "grind.lock" file, re-generate on add/remove or mismatch on "install"
  - [x] ✅ Correctly handle super POM via `<parent>` recursive walk,
  - [x] ✅ Handle `<dependencyManagement>` resolution and order of inheritence
  - [x] ✅ Handle cyclic dependency checks
  - [x] ✅ Handle BOM import
  - [x] ✅ Handle `<optional>` dependencies
  - [x] ✅ Handle property interpolation
  - [ ] ⚠️ Handle exclusions
  - [ ] ⚠️ Handle version ranges and specifiers e.g `>=, <, -` etc _(this will be a fairly massive undertaking!)_
- [x] ✅ Compile and build Jar file
- [x] ✅ Compile and run Project
- [x] ✅ Run a specific task as defined in the `grind.yml` manifest
- [x] ✅ List all available custom tasks
- [x] ✅ Add a dependency
- [x] ✅ Remove a dependency
- [x] ✅ Testing: using custom test runner [TestTube](https://github.com/AnharHussainMiah/TestTube) built ironically using Grind! _(with built in package integrity checks)_
- [x] 🧪 Experimental "fat jar" aka `uberjar`
- [ ] 🔨 Split out test dependency when adding/removing _(have separate folder)_ e.g `libs-test`
- [ ] 🔨 Implement version pinning

### 🎉 MILESTONE:

`grind` has been tested against the original famous [Pet Clinic Spring Demo](https://github.com/spring-projects/spring-petclinic). We re-created this project using `grind` and as `v0.7.4` able to fully compile, run, and even bundle a fat jar! here is the grind version of the [Pet Clinic](https://github.com/AnharHussainMiah/PetClinicApplication/)

## Long Term Goals

- [ ] Manage Java SDK versions a bit like "Node Version Manager" or "rustup"
- [ ] Other Repositories other than Maven or Support custom/private repos?
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

        - "Java builds, without the headache"
                    v0.7.4


Usage: grind <COMMAND>

Commands:
  new        Scaffolds a new Java project with a grind.yml file
  install    Download all the external libraries and dependencies as defined in the grind.yml
  build      Compile the project and builds a jar file
  run        Compile and run the project
  add        Adds a dependency to the project's grind.yml
  remove     Removes a dependency from the project's grind.yml
  task       Run a custom task as defiend in the grind.yml, e.g grind task clean
  integrity  Create the integrity file or validate one for plugins/packages
  test       Run Tests
  bundle     Packages compiled classes and all dependency jars into a single runnable JAR, also known as a "Fat Jar" or "Uberjar"
  help       Print this message or the help of the given subcommand(s)

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

To install all your projects dependencies that are defined in the `grind.yml` file, simply invoke the following:

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

To compile and run your project, simply invoke the following:

```bash
grind run
```

The run command can include the `profile` (see section 9) as well as command line arguments to forward to your compiled application e.g:

```bash
grind run dev arg1 arg2 arg2
```

In the above, the first argument is the "profile" and the remaining arguments are passed through as is. If **no profile matches**, then they're all treated as arguments.

### 6. Compile and Package up a final Jar executable

To build your production `jar` simply invoke the following:

```bash
grind build
```

Your compiled and package `jar` will be available in the `build/` folder, however keep in mind because this is not
a "fat jar" or "uber jar", your dependecies which are located in the `libs/` folder would need to be relative to where the `jar` is.

So for production you would need to include both your `jar` file as well as the `libs/` folder.

### 7. Optional run custom tasks

Much like the tasks that can be set in the `package.json` in `npm`, grind also has a similar feature, you can list the current available tasks as follows:

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

### 8. Running Tests

Assuming you have written your test classes _(and have included the Junit dependency)_, to run a specific test or tests simply invoke:

```shell
# run a single test
grind test com.example.Foo
# run multiple tests
grind test com.example.FooTest com.example.BarTest com.example.BazTest
# run all tests
grind test
```

This will automatically download the test plugin and install it, unpack it and verify the bundle, then compile all the test classes and then run the tests, and finally write the XML reports to disk under `reports`

```shell
🌎 Downloading TestTube plugin...
🗜️ Extracting TestTube plugin...
✅ Extraction complete!
🔍 Verifying files in directory: plugins/TestTube
[OK] 19e52f8ac17a67f625a337e9e38622a6 | libs/org.junit.platform_junit-platform-launcher_1.13.0-M3.jar
[OK] 8c7de3f82037fa4a2e8be2a2f13092af | libs/org.apiguardian_apiguardian-api_1.1.2.jar
[OK] 005914a884db365bfefd3b36086703d3 | libs/org.junit.platform_junit-platform-commons_1.13.0-M3.jar
[OK] 45c09ab04dd09ef6afbda47eb0ae31a5 | libs/org.junit.platform_junit-platform-reporting_1.13.0-M3.jar
[OK] 03c404f727531f3fd3b4c73997899327 | libs/org.opentest4j_opentest4j_1.3.0.jar
[OK] 58ab9d57ded624201ae0319ffda5995c | libs/org.junit.jupiter_junit-jupiter-api_5.13.0-M3.jar
[OK] c502ae6080594b6437ef39c4a47038fc | libs/org.junit.jupiter_junit-jupiter-engine_5.13.0-M3.jar
[OK] ed04fe87e33c5accbf0a01c1aa9bdafa | libs/org.opentest4j.reporting_open-test-reporting-tooling-spi_0.2.3.jar
[OK] f70eba72906c90378b0cdc5b27831b8a | libs/org.junit.platform_junit-platform-engine_1.13.0-M3.jar
[OK] ad8fe13cd287bdfe00aa52576b33d5f1 | TestTube.jar

📄 Summary:
  Total files checked : 10
  Missing files       : 0
  Hash mismatches     : 0
✅ All files passed integrity check.
✅ Integrity check passed.
==> 🔨 compiling project [HelloWorld]...
==> 🔨 compiling tests for [HelloWorld]...
Adding Dynamic Class -> com.example.HelloWorldTest
========== Test Summary ==========
Total tests:      1
✅ Passed:        1
❌ Failed:        0
⏩ Skipped:       0
🚫 Aborted:       0
==================================

✅ All tests passed.

📄 XML reports written to: /home/anhar/Documents/Projects/grind/HelloWorld/reports
```

### 9. Using "Profiles"

Grind supports build and run `profiles` that can be defined in the `grind.yml` for example:

```yaml
profiles:
  dev:
    flags:
      - -parameters
    envs:
      API_KEY: "xxx"
      DATABASE_URL: "postgres://localhost/dev_db"
  stage:
    envs:
      API_KEY: "yyy"
      DATABASE_URL: "postgres://staging.server/stage_db"
  prod:
    flags:
      - -parameters
    envs:
      API-KEY: "zzz"
      DATABASE_URL: "postgres://prod.server/prod_db"
```

Each profile can contain optional compiler `flags` as well as `envs` that are environment variables, to use a profile, simply add the profile name after the `run` or `build`, for example:

```shell
grind run dev
grind build prod
```

### Dependencies

Grind assumes the following are already installed on your machine:

- Bash
- Java

## ~~No FAT Jars!~~ Fat Jars!

I was deciding on if `grind` should be able to create "fat Jars", but then I realised that in modern development we end up creating container images. In that context creating a fat jar doesn't make all that sense! AND you potentially lose out on caching!

Instead of creating a fat jar, just copy the `libs` folder, you'll end up with a single container image anyway, but with the advantage of proper caching layers meaning, next time your update your images, it will only update the actual application layer which could be just a few kilobytes vs hundreds of megabytes!

**EDIT:** We now have as of version `v0.7.4` experimental support for fat jars, this can be done simply be using the `bundle` command, this also supports custom compiler flag options via the profiles so for example `grind bundle prod` etc, _but here be dragons!_, while it may work for most projects, if you have some really deep dependencies I'm not sure if the merging logic is 100% at the moment.

## Visual Studio Code Support

Make sure you have installed the official Microsoft _"Extension Pack for Java"_ extension, `grind` will automatically generate the correct settings for it under `.vscode/settings.json` when a new project is created, so you can just open the project and start coding!

For other plugins, or editors and LSP, you will need to configure the `classpath` as well as the source paths e.g:

- classpath, set to `libs/`
- source path set as `src/java/main`

## No Windows Support.

Techinally `grind` could be made to support Windows _(switching out bash for poweshell, and minor changes in classpaths)_, but I don't have the energy to make that happen! It's open source and I would massively welcome all contributions for any feature(s) that you would really like to see.

## Design Philosophy

I'm following the `YAGNI`, `KISS`, and the _"deliver the simplest solution that works."_ approach to software design. I also do not like lots of nested folders and endless abstractions, so I try to organise things into their own simple
module and keep things as "flat" as possible. I believe keeping the indirection as low as possible makes it easier to reason about the code.

## Contributions

I'm open to contributions, suggestions, and ideas, etc but please be kind!

If you've found this project helpful, please leave a 🌟 or help spread the word!
