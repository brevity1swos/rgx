"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.setOptionValues = setOptionValues;
function setOptionValues(options, object) {
    for (const key of Object.keys(object)) {
        if (options[key] !== undefined) {
            object[key] = options[key];
        }
    }
    return object;
}
