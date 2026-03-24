/**
 * @fileoverview "table reporter.
 * @author Gajus Kuizinas <gajus@gajus.com>
 * @copyright 2016 Gajus Kuizinas <gajus@gajus.com>. All rights reserved.
 */
import type { TextlintResult } from "@textlint/types";
import { FormatterOptions } from "../FormatterOptions.js";
declare function formatter(report: TextlintResult[], options: FormatterOptions): string;
export default formatter;
//# sourceMappingURL=table.d.ts.map