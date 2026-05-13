# Building the Java Wrapper

The Java Wrapper allows PicoLimbo to be used as a plugin for proxy servers like BungeeCord and Velocity, or as a standalone Java application. While the official releases are built automatically, you may want to build the wrapper manually for development or custom distributions.

## Prerequisites

Before starting, ensure you have the following installed:

- **Rust**: Including the target toolchain for your platform.
- **Java 21**: The wrapper requires JDK 21 to compile.
- **Git**: To clone the repository.

## Build Steps

### 1. Compile the Rust Library
The Java wrapper relies on a compiled Rust library (`cdylib`). You must build this library first.

> [!TIP]
> For local testing, you only need to build the library for your current platform. The official release workflow automatically builds and bundles all native libraries for all supported platforms.

Run the following command based on your target platform:

**Linux (x86_64):**
```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu --lib
```

**Linux (AArch64):**
```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu --lib
```

**macOS (Apple Silicon):**
```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin --lib
```

**Windows (x86_64):**
```bash
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --lib
```

### 2. Place the Library in the Wrapper Resources
The Gradle build process bundles the native libraries into the JAR. You must copy the compiled library to the correct resource directory.

| Platform           | Source Path                                                     | Destination Path                                          |
|:-------------------|:----------------------------------------------------------------|:----------------------------------------------------------|
| **Linux x86_64**   | `target/x86_64-unknown-linux-gnu/release/libpico_limbo_lib.so`  | `java_wrapper/wrapper/src/main/resources/linux/x86_64/`   |
| **Linux AArch64**  | `target/aarch64-unknown-linux-gnu/release/libpico_limbo_lib.so` | `java_wrapper/wrapper/src/main/resources/linux/aarch64/`  |
| **macOS AArch64**  | `target/aarch64-apple-darwin/release/libpico_limbo_lib.dylib`   | `java_wrapper/wrapper/src/main/resources/macos/aarch64/`  |
| **Windows x86_64** | `target/x86_64-pc-windows-msvc/release/pico_limbo_lib.dll`      | `java_wrapper/wrapper/src/main/resources/windows/x86_64/` |

**Example for Linux x86_64:**
```bash
mkdir -p java_wrapper/wrapper/src/main/resources/linux/x86_64
cp target/x86_64-unknown-linux-gnu/release/libpico_limbo_lib.so java_wrapper/wrapper/src/main/resources/linux/x86_64/
```

### 3. Build the JAR
Once the libraries are in place, use the Gradle wrapper to build the Java project.

```bash
cd java_wrapper
./gradlew build
```

## Verification

After the build completes, you can find the resulting JAR file at:
`java_wrapper/wrapper/build/libs/wrapper.jar`

To verify the installation, you can try running the standalone version or loading it into your proxy server of choice.

## Troubleshooting

- **JAVA_HOME not set**: If Gradle fails with a `JAVA_HOME` error, ensure your JDK 21 installation is correctly configured in your environment variables.
- **Wrong Library Architecture**: If the wrapper crashes with an `UnsatisfiedLinkError`, double-check that the library in `src/main/resources` matches the architecture of the machine running the JAR.
