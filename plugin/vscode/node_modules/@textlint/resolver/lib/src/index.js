"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.clearHooks = exports.dynamicImport = exports.registerImportHook = exports.tryResolve = exports.registerResolveHook = void 0;
const url = __importStar(require("node:url"));
const node_path_1 = __importDefault(require("node:path"));
const resolveHooks = [];
const importHooks = [];
/**
 * Register Resolver Hook
 * Hook can return resolved URL
 * if hooks pass through, it should return `undefined` instead of object
 * @param hook
 */
const registerResolveHook = (hook) => {
    resolveHooks.push(hook);
};
exports.registerResolveHook = registerResolveHook;
/**
 * Try to resolve package name
 * if `packageName` is found, return resolved absolute path.
 * if `packageName` is not found, return `undefined`
 * @param packageName
 * @param context
 */
const tryResolve = (packageName, context) => {
    try {
        for (const hook of resolveHooks) {
            const result = hook(packageName, context);
            // Skip if hook return undefined from hook
            if (!result) {
                continue;
            }
            if (result === null || result === void 0 ? void 0 : result.url) {
                return result.url;
            }
        }
        // TODO: consider using import.meta.resolve(packageName) for future ESM support
        // import.meta.resolve is supported in Node.js 20.6.0+
        return require.resolve(packageName);
    }
    catch (_a) {
        return undefined;
    }
};
exports.tryResolve = tryResolve;
/**
 * Register Import Hook
 * @param hook
 */
const registerImportHook = (hook) => {
    importHooks.push(hook);
};
exports.registerImportHook = registerImportHook;
// Windows's path require to convert file://
// https://github.com/secretlint/secretlint/issues/205
const convertToFileUrl = (filePath) => {
    if (filePath.startsWith("file://")) {
        return filePath;
    }
    return url.pathToFileURL(filePath).href;
};
/**
 * dynamic import() with hooks
 * @param specifier file path or package name
 * @param context
 */
const dynamicImport = async (specifier, context) => {
    for (const hook of importHooks) {
        const result = await hook(specifier, context);
        if (result) {
            return result;
        }
    }
    // if the `specifier` is not absolute path, it should be package name
    if (!node_path_1.default.isAbsolute(specifier)) {
        return {
            exports: await import(specifier),
        };
    }
    return {
        exports: await import(convertToFileUrl(specifier)),
    };
};
exports.dynamicImport = dynamicImport;
/**
 * Clear all hooks
 */
const clearHooks = () => {
    resolveHooks.length = 0;
    importHooks.length = 0;
};
exports.clearHooks = clearHooks;
//# sourceMappingURL=index.js.map