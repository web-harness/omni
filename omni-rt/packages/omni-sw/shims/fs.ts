import { fs } from "../src/zenfs.js";

export const promises = fs.promises;
export const access = (...args: Parameters<typeof fs.access>) => fs.access(...args);
export const chmod = (...args: Parameters<typeof fs.chmod>) => fs.chmod(...args);
export const copyFile = (...args: Parameters<typeof fs.copyFile>) => fs.copyFile(...args);
export const existsSync = (...args: Parameters<typeof fs.existsSync>) => fs.existsSync(...args);
export const lstat = (...args: Parameters<typeof fs.lstat>) => fs.lstat(...args);
export const mkdir = (...args: Parameters<typeof fs.mkdir>) => fs.mkdir(...args);
export const readFile = (...args: Parameters<typeof fs.readFile>) => fs.readFile(...args);
export const readdir = (...args: Parameters<typeof fs.readdir>) => fs.readdir(...args);
export const readlink = (...args: Parameters<typeof fs.readlink>) => fs.readlink(...args);
export const rename = (...args: Parameters<typeof fs.rename>) => fs.rename(...args);
export const rm = (...args: Parameters<typeof fs.rm>) => fs.rm(...args);
export const stat = (...args: Parameters<typeof fs.stat>) => fs.stat(...args);
export const symlink = (...args: Parameters<typeof fs.symlink>) => fs.symlink(...args);
export const unlink = (...args: Parameters<typeof fs.unlink>) => fs.unlink(...args);
export const writeFile = (...args: Parameters<typeof fs.writeFile>) => fs.writeFile(...args);

export default {
  ...fs,
  promises,
};
