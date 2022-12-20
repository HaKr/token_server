import { Result } from "./std/result.ts";

export abstract class Failure {
  constructor(public reason?: unknown) {}
  toString(): string {
    return this.reason instanceof Failure
      ? this.reason.toString()
      : `${
        this.constructor.name.replace(/(\w)([A-Z])/g, (_sub, ...args) => {
          const [a, b] = args as string[];
          return `${a == "_" ? "" : a} ${b.toLowerCase()}`;
        })
      }${this.reason == undefined ? "" : `: ${this.reason}`}`;
  }
}

export type SimulationResult = Result<void, SimulationFailed>;
export class SimulationFailed extends Failure {}

export class SimulationAborted extends SimulationFailed {}
export class MissingToken extends SimulationFailed {}
