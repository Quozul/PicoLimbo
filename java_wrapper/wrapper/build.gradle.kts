plugins {
    application
    id("com.modrinth.minotaur") version "2.9.0"
}

val picoLimboCargoToml = rootProject.file("../pico_limbo/Cargo.toml")
val picoLimboVersion = Regex("(?m)^version\\s*=\\s*\"([^\"]+)\"")
    .find(picoLimboCargoToml.readText())
    ?.groupValues
    ?.get(1)
    ?: error("Could not read the package version from ${picoLimboCargoToml.path}")

version = picoLimboVersion

repositories {
    mavenCentral()

    maven {
        name = "papermc"
        url = uri("https://repo.papermc.io/repository/maven-public/")
    }

    maven {
        name = "bungeecord-repo"
        url = uri("https://central.sonatype.com/repository/maven-snapshots/")
    }
}

dependencies {
    implementation(libs.jna)
    compileOnly(libs.bungeecord)
    compileOnly(libs.velocity)
    annotationProcessor(libs.velocity)
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(25)
    }
}

application {
    mainClass = "dev.quozul.Standalone"
}

tasks.jar {
    archiveFileName = "pico_limbo_java_wrapper.jar"

    manifest {
        attributes["Main-Class"] = application.mainClass
    }

    from(sourceSets.main.get().output)

    dependsOn(configurations.runtimeClasspath)
    from({
        configurations.runtimeClasspath.get().filter { it.name.endsWith("jar") }.map { zipTree(it) }
    })

    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
}

val libName = project.property("libName") as String

val templateSource = file("src/main/templates")
val templateDest: Provider<Directory> = layout.buildDirectory.dir("generated/sources/templates")
val generateTemplates = tasks.register<Copy>("generateTemplates") {
    val props = mapOf("libName" to libName)
    inputs.properties(props)

    from(templateSource)
    into(templateDest)
    expand(props)
}

sourceSets {
    main {
        java {
            srcDir(generateTemplates.map { it.outputs })
        }
    }
}

modrinth {
    token.set(System.getenv("MODRINTH_TOKEN"))
    projectId.set("picolimbo-java-wrapper")
    versionNumber.set(picoLimboVersion)
    versionType.set(
        when {
            picoLimboVersion.contains("alpha", ignoreCase = true) -> "alpha"
            picoLimboVersion.contains("beta", ignoreCase = true) -> "beta"
            else -> "release"
        }
    )
    uploadFile.set(tasks.jar)
    changelog.set("Check the changelog at https://github.com/Quozul/PicoLimbo/blob/master/CHANGELOG.md")
    gameVersions.addAll(
        "1.7.2", "1.7.3", "1.7.4", "1.7.5", "1.7.6", "1.7.7", "1.7.8", "1.7.9", "1.7.10",
        "1.8", "1.8.1", "1.8.2", "1.8.3", "1.8.4", "1.8.5", "1.8.6", "1.8.7", "1.8.8", "1.8.9",
        "1.9", "1.9.1", "1.9.2", "1.9.3", "1.9.4",
        "1.10", "1.10.1", "1.10.2",
        "1.11", "1.11.1", "1.11.2",
        "1.12", "1.12.1", "1.12.2",
        "1.13", "1.13.1", "1.13.2",
        "1.14", "1.14.1", "1.14.2", "1.14.3", "1.14.4",
        "1.15", "1.15.1", "1.15.2",
        "1.16", "1.16.1", "1.16.2", "1.16.3", "1.16.4", "1.16.5",
        "1.17", "1.17.1",
        "1.18", "1.18.1", "1.18.2",
        "1.19", "1.19.1", "1.19.2", "1.19.3", "1.19.4",
        "1.20", "1.20.1", "1.20.2", "1.20.3", "1.20.4", "1.20.5", "1.20.6",
        "1.21", "1.21.1", "1.21.2", "1.21.3", "1.21.4", "1.21.5", "1.21.6", "1.21.7", "1.21.8", "1.21.9", "1.21.10", "1.21.11",
        "26.1", "26.1.1", "26.1.2",
        "26.2",
        "26.3"
    )
    loaders.addAll("velocity", "bungeecord", "java-agent")
}
