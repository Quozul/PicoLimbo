import {
    writeFile,
    mkdtemp,
    copyFile,
    rm,
    mkdir,
    readdir,
    opendir,
    readFile,
} from "node:fs/promises";
import { join, dirname } from "node:path";
import { exec } from "node:child_process";
import { cleanDataDirectory } from "./clean/cleanData.ts";
import { downloadServerJars } from "./fetch/serverJar.ts";
import { fileExists } from "./fetch/fileExists.ts";

const execute = async (command: string, cwd: string): Promise<string> =>
    new Promise((resolve, reject) => {
        exec(command, { cwd }, (error, stdout, stderr) => {
            if (error) {
                return reject(error);
            }
            if (stderr) {
                return reject(stderr);
            }
            return resolve(stdout);
        });
    });

const SUPPORTED_VERSIONS = [
    "1.21.9-pre1",
    "1.21.7",
    "1.21.6",
    "1.21.5",
    "1.21.4",
    "1.21.2",
    "1.21",
    "1.20.5",
    "1.20.3",
    "1.20.2",
    "1.20",
    "1.19.4",
    "1.19.3",
    "1.19.1",
    "1.19",
];

(async () => {
    const serverJarDirectory = "servers";
    const jarFiles = await downloadServerJars(
        SUPPORTED_VERSIONS,
        serverJarDirectory,
    );

    for (const version of jarFiles) {
        const outputDirectory = join(
            process.cwd(),
            "generated",
            `V${version.version.replaceAll(".", "_")}`,
        );

        if (await fileExists(outputDirectory)) {
            console.log(`Skipping version ${version.version}`);
            continue;
        }

        // Run the server to output the files
        const generatedDirectory = await mkdtemp(
            `/tmp/generated_${version.version}`,
        );
        const command = `java -DbundlerMainClass=net.minecraft.data.Main -jar ${version.fileName} --reports --server --output ${generatedDirectory}`;
        try {
            await execute(command, serverJarDirectory);
        } catch (e) {
            console.error(
                `An error occurred while processing version ${version.fileName}:`,
                e,
            );
        }
        console.log(`Generated ${version.version}: ${version.path}`);

        // Copy the generated data
        const dataDirectory = await move(
            generatedDirectory,
            outputDirectory,
            "data",
        );
        const reportsDirectory = await move(
            generatedDirectory,
            outputDirectory,
            "reports",
        );

        // Cleanup
        await cleanDataDirectory(dataDirectory);
        const wolfVariant = join(dataDirectory, "minecraft", "wolf_variant");
        if (await fileExists(wolfVariant)) {
            await cleanWolfVariants(wolfVariant);
        }
        await cleanReportsDirectory(reportsDirectory);
        await rm(generatedDirectory, { recursive: true, force: true });
    }
})();

const move = async (
    from: string,
    to: string,
    subdir: string,
): Promise<string> => {
    const destination = join(to, subdir);
    await copyDir(join(from, subdir), destination);
    return destination;
};

async function copyDir(src: string, dest: string): Promise<void> {
    const entries = await readdir(src, {
        recursive: true,
        withFileTypes: true,
    });

    for (const entry of entries) {
        const srcPath = join(entry.parentPath, entry.name);
        const destPath = srcPath.replace(src, dest);
        const destDir = dirname(destPath);

        if (entry.isFile()) {
            await mkdir(destDir, { recursive: true });
            await copyFile(srcPath, destPath);
        }
    }
}

async function cleanWolfVariants(path: string): Promise<void> {
    // Replace "#minecraft:is_" with "minecraft:" in wolf_variant
    const wolfVariantDirectory = await opendir(path);
    for await (const dirent of wolfVariantDirectory) {
        const direntPath = join(path, dirent.name);
        const fileContents = await readFile(direntPath, "utf8");
        await writeFile(
            direntPath,
            fileContents.replace("#minecraft:is_", "minecraft:"),
        );
    }
}

async function cleanReportsDirectory(path: string): Promise<void> {
    // Only keep the packets.json file
    const dir = await opendir(path);
    for await (const dirent of dir) {
        if (dirent.name !== "packets.json" && dirent.name !== "blocks.json" && dirent.name !== "registries.json") {
            const direntPath = join(path, dirent.name);
            await rm(direntPath, { recursive: true, force: true });
        }
    }
}
