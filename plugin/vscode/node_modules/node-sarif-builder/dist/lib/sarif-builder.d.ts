import { Log, Run } from 'sarif';
import { LogOptions } from '../types/node-sarif-builder';
import { SarifRunBuilder } from './sarif-run-builder';
export declare class SarifBuilder {
    log: Log;
    constructor(options?: LogOptions);
    addRun(sarifRunBuilder: SarifRunBuilder): void;
    generateSarifFileSync(file: string): void;
    generateSarifFile(file: string): Promise<void>;
    buildSarifOutput(): Log;
    buildSarifJsonString(options?: {
        indent: boolean;
    }): string;
    completeRunFields(run: Run): Run;
}
