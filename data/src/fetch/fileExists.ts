import { stat } from "node:fs/promises";

export const fileExists = async (path: string) =>
    !!(await stat(path).catch((_) => false));
