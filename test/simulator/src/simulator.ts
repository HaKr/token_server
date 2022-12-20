import { Meta } from "./api.ts";
import { MissingToken, SimulationAborted } from "./error.ts";
import { SimulationFailed } from "./error.ts";
import { Logging } from "./logging.ts";
import { metadata_collection } from "./mock/metadata_collection.js";
import { Scheduler } from "./scheduler.ts";
import { Session } from "./session.ts";
import { Err, Ok, Result } from "./std/result.ts";
import { Task } from "./tasks.ts";
import { ClientError, NoConnection, TokenClient } from "./token_client.ts";

type SimulationResult = Result<boolean, SimulationFailed>;
type SimulationTaskResult = Result<void, SimulationFailed>;
export type TaskExecutor = (
  session: Session,
) => Promise<SimulationTaskResult>;

export class Simulator {
  static LOGGER = Logging.for(Simulator.name);
  static taskMap: { [key: string]: TaskExecutor } = {
    [Task.Create]: Simulator.prototype.create,
    [Task.Update]: Simulator.prototype.update,
    [Task.UpdateWithError]: Simulator.prototype.update_with_xml,
    [Task.Refresh]: Simulator.prototype.refresh,
    [Task.Remove]: Simulator.prototype.remove,
  };

  static clientErrorToSimulationResult(
    err: ClientError,
  ): SimulationFailed {
    if (err instanceof NoConnection) {
      return new SimulationAborted("server unavailable");
    }
    return new SimulationFailed(err);
  }

  client = new TokenClient();
  created = Date.now();
  scheduler;

  constructor(
    public name: string,
    include_errors: boolean,
    random_wait: number,
  ) {
    let index = 0;
    this.scheduler = new Scheduler(random_wait > 0);
    for (const assignment of metadata_collection) {
      assert(
        assignment == null || typeof assignment == "object",
        "Illegal input format",
      );
      const delays = generate_delays(random_wait);
      let when = 0;
      const session = new Session(assignment as Meta);
      for (
        const task of [Task.Create, Task.Update].concat(
          include_errors && index == 1 ? Task.Remove : [],
        ).concat(
          include_errors && index == 3 ? Task.UpdateWithError : [],
        )
          .concat(Task.Refresh)
      ) {
        when += delays.next().value!;
        this.scheduler.schedule(
          when,
          task,
          session, // deliberately surpassing check for correct input data
        );
      }
      index++;
    }
  }

  async run(): Promise<SimulationResult> {
    const iter = this.scheduler.iter();

    let result;
    do {
      result = await (await iter.next()).async_map_or_else<SimulationResult>(
        () => Promise.resolve(Ok<boolean, SimulationFailed>(false)),
        async (todo) => {
          const info = `${todo.session}  ${todo.task}`;
          return (await Simulator.taskMap[todo.task].call(this, todo.session))
            .map_or_else(
              (err) => {
                if (err instanceof SimulationAborted) {
                  return Err(err);
                } else {
                  this.log_failure(info, err.toString());
                }
                return Ok(true);
              },
              () => {
                this.log_success(info);
                return Ok(true);
              },
            );
        },
      );
    } while (result.unwrap_or(false));

    return result.map((looping) => !looping);
  }

  shutdownServer() {
    return this.client.shutdown();
  }

  protected async create(session: Session) {
    return (await this.client.create_token(session.meta))
      .map((token) => session.create(token)).map_err(
        Simulator.clientErrorToSimulationResult,
      );
  }

  protected update(session: Session): Promise<SimulationTaskResult> {
    return this.update_task(session, false);
  }

  protected update_with_xml(session: Session): Promise<SimulationTaskResult> {
    return this.update_task(session, true);
  }

  protected update_task(
    session: Session,
    forceMediaError: boolean,
  ): Promise<SimulationTaskResult> {
    return session.token_or_else<SimulationFailed>(() => new MissingToken())
      .async_map_or_else<SimulationTaskResult>(
        Err,
        async (token) =>
          (await this.client.update_token(
            token,
            { updatedAt: Date.now() },
            forceMediaError,
          ))
            .map((update_result) =>
              session.update(update_result.token, update_result.meta)
            ).map_err(Simulator.clientErrorToSimulationResult),
      );
  }

  protected refresh(session: Session): Promise<SimulationTaskResult> {
    return session.token_or_else<SimulationFailed>(() => new MissingToken())
      .async_map_or_else<SimulationTaskResult>(
        Err,
        async (token) =>
          (await this.client.update_token(token))
            .map((update_result) =>
              session.update(update_result.token, update_result.meta)
            ).map_err(Simulator.clientErrorToSimulationResult),
      );
  }

  protected remove(session: Session): Promise<SimulationTaskResult> {
    return session.token_or_else<SimulationFailed>(() => new MissingToken())
      .async_map_or_else<SimulationTaskResult>(
        Err,
        async (token) =>
          (await this.client.delete_token(token))
            .map(() => session.clear_token())
            .map_err(Simulator.clientErrorToSimulationResult),
      );
  }

  private log_success(what: string) {
    Simulator.LOGGER.info(`${this.formatLabel()} ${what} succeeded`);
  }

  private log_failure(what: string, why: string) {
    Simulator.LOGGER.error(
      `${this.formatLabel()} ${what} failed${
        why.length > 0 ? `: ${maxWidth(why, 100)}` : ""
      }`,
    );
  }

  private formatLifetime(lifetime: number, pad: number): string {
    return `${lifetime}`.padStart(pad, "0");
  }

  private formatLabel(): string {
    return `[${
      this.formatLifetime(Date.now() - this.created, 5)
    }ms ${this.name}]`;
  }
}

function maxWidth(str: string, width: number) {
  if (str.length <= width) return str;
  const half = (width - 5) / 2;
  return `${str.slice(0, half)} ... ${str.slice(-half)}`;
}

function* generate_delays(maxSeconds: number) {
  yield 0;
  while (true) {
    yield Math.round(Math.random() * maxSeconds * 1000);
  }
}

function assert(predicate: boolean, msg?: string | undefined): void | never {
  if (!predicate) throw new Error(msg || "Assertion failed");
}
