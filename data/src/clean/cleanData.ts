import { opendir, rm } from "node:fs/promises";
import { join } from "node:path";

/**
 * Recursively cleans the given directory.
 *
 * If a directory’s relative path exactly matches one of the allowed paths,
 * that directory (and its contents) is preserved.
 * If the directory is not exactly allowed but is a parent of an allowed path,
 * then its children are processed so that only the allowed subdirectories remain.
 *
 * @param directory The absolute path of the current directory.
 * @param relPath The relative path from the "minecraft" folder.
 * @param toKeep List of relative directory paths that should be kept.
 */
async function cleanDirectory(
    directory: string,
    relPath: string,
    toKeep: string[],
): Promise<void> {
    const dirHandle = await opendir(directory);
    for await (const entry of dirHandle) {
        // Compute the entry's relative path (e.g. "worldgen/biome")
        const entryRelPath = relPath ? join(relPath, entry.name) : entry.name;
        const fullPath = join(directory, entry.name);

        if (entry.isDirectory()) {
            // If the directory is exactly one of the allowed ones, leave it and its contents intact.
            if (toKeep.includes(entryRelPath)) {
                // Skip cleaning this allowed directory
                continue;
            }
            // If this directory is not explicitly allowed but is a parent of an allowed directory,
            // we need to traverse into it so we can remove any siblings that aren’t allowed.
            if (
                toKeep.some((allowed) => allowed.startsWith(`${entryRelPath}/`))
            ) {
                await cleanDirectory(fullPath, entryRelPath, toKeep);
            } else {
                // Otherwise, this directory is not allowed at all, so remove it.
                await rm(fullPath, { recursive: true, force: true });
            }
        } else {
            // Remove any files (if needed). If you want to keep certain files inside allowed directories,
            // adjust the logic accordingly.
            await rm(fullPath, { force: true });
        }
    }
}

/**
 * Cleans the "minecraft" folder so that only the registries listed in `toKeep` (and any
 * parent directories necessary for their path) remain.
 *
 * For directories explicitly listed in `toKeep`, their entire contents are preserved.
 *
 * @param path The base path that contains the "minecraft" folder.
 * @param toKeep List of registry directories (relative to "minecraft") that should be kept.
 */
export async function cleanDataDirectory(
    path: string,
    toKeep: string[] = REGISTRIES_TO_SEND,
): Promise<void> {
    const minecraftDir = join(path, "minecraft");
    await cleanDirectory(minecraftDir, "", toKeep);
}

const REGISTRIES_TO_SEND = [
    "damage_type",
    "dimension_type",
    "painting_variant",
    "wolf_variant",
    "worldgen/biome",
    // The following registries were added in 1.21.5
    "cat_variant",
    "chicken_variant",
    "cow_variant",
    "frog_variant",
    "pig_variant",
    "wolf_sound_variant",
    // The following registries were added in 1.21.11
    "zombie_nautilus_variant",
];
