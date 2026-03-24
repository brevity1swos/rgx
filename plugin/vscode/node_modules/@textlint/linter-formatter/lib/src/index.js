// LICENSE : MIT
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.loadFormatter = loadFormatter;
exports.getFormatterList = getFormatterList;
const module_interop_1 = require("@textlint/module-interop");
const node_fs_1 = __importDefault(require("node:fs"));
const node_path_1 = __importDefault(require("node:path"));
const resolver_1 = require("@textlint/resolver");
const debug_1 = __importDefault(require("debug"));
// formatter
const checkstyle_js_1 = __importDefault(require("./formatters/checkstyle.js"));
const compact_js_1 = __importDefault(require("./formatters/compact.js"));
const github_js_1 = __importDefault(require("./formatters/github.js"));
const jslint_xml_js_1 = __importDefault(require("./formatters/jslint-xml.js"));
const json_js_1 = __importDefault(require("./formatters/json.js"));
const junit_js_1 = __importDefault(require("./formatters/junit.js"));
const pretty_error_js_1 = __importDefault(require("./formatters/pretty-error.js"));
const stylish_js_1 = __importDefault(require("./formatters/stylish.js"));
const table_js_1 = __importDefault(require("./formatters/table.js"));
const tap_js_1 = __importDefault(require("./formatters/tap.js"));
const unix_js_1 = __importDefault(require("./formatters/unix.js"));
const builtinFormatterList = {
    checkstyle: checkstyle_js_1.default,
    compact: compact_js_1.default,
    "github": github_js_1.default,
    "jslint-xml": jslint_xml_js_1.default,
    json: json_js_1.default,
    junit: junit_js_1.default,
    "pretty-error": pretty_error_js_1.default,
    stylish: stylish_js_1.default,
    table: table_js_1.default,
    tap: tap_js_1.default,
    unix: unix_js_1.default
};
const builtinFormatterNames = Object.keys(builtinFormatterList);
const debug = (0, debug_1.default)("textlint:@textlint/linter-formatter");
const isFormatterFunction = (formatter) => {
    return typeof formatter === "function";
};
async function loadFormatter(formatterConfig) {
    var _a;
    const formatterName = formatterConfig.formatterName;
    debug(`formatterName: ${formatterName}`);
    if (builtinFormatterNames.includes(formatterName)) {
        return {
            format(results) {
                return builtinFormatterList[formatterName](results, formatterConfig);
            }
        };
    }
    let formatter;
    let formatterPath;
    if (node_fs_1.default.existsSync(formatterName)) {
        formatterPath = formatterName;
    }
    else if (node_fs_1.default.existsSync(node_path_1.default.resolve(process.cwd(), formatterName))) {
        formatterPath = node_path_1.default.resolve(process.cwd(), formatterName);
    }
    else {
        const pkgPath = (0, resolver_1.tryResolve)(`textlint-formatter-${formatterName}`, {
            parentModule: "linter-formatter"
        }) ||
            (0, resolver_1.tryResolve)(formatterName, {
                parentModule: "linter-formatter"
            });
        if (pkgPath) {
            formatterPath = pkgPath;
        }
    }
    if (!formatterPath) {
        throw new Error(`Could not find formatter ${formatterName}`);
    }
    try {
        const mod = (_a = (0, module_interop_1.moduleInterop)((await (0, resolver_1.dynamicImport)(formatterPath, {
            parentModule: "linter-formatter"
        })).exports)) === null || _a === void 0 ? void 0 : _a.default;
        if (!isFormatterFunction(mod)) {
            throw new Error(`formatter should export function, but ${formatterPath} exports ${typeof mod}`);
        }
        formatter = mod;
    }
    catch (ex) {
        throw new Error(`Could not find formatter ${formatterName}
${ex}`);
    }
    return {
        format(results) {
            return formatter(results, formatterConfig);
        }
    };
}
function getFormatterList() {
    return builtinFormatterNames.map((name) => {
        return {
            name
        };
    });
}
//# sourceMappingURL=index.js.map