import { Logging } from "./logging.ts";
import { Session } from "./session.ts";
import { None, optionFrom, Some } from "./deps.ts";
import { Task } from "./tasks.ts";
import { ToDo } from "./todo.ts";

export class Scheduler {
  static LOGGER = Logging.for(Scheduler.name);
  tasks: ToDo[] = [];
  sleep: (ms: number) => Promise<void>;

  constructor(private doSleep: boolean) {
    this.sleep = doSleep
      ? (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))
      : (_: number) => Promise.resolve();
  }

  schedule(when: number, task: Task, session: Session) {
    this.tasks.push(new ToDo(session, task, when > 0 ? Date.now() + when : 0));
  }

  iter() {
    this.tasks.sort((a, b) => a.when - b.when);
    const iterator = this.tasks[Symbol.iterator]();

    return {
      next: () =>
        optionFrom((async () => {
          let todo;
          do {
            todo = iterator.next();
            if (todo.done) return Promise.resolve(None<ToDo>());
          } while (!todo.value.shouldExecute());
          const now = Date.now();
          if (this.doSleep && todo.value.when > now) {
            const ms = todo.value.when - now;
            Scheduler.LOGGER.trace(
              `Wait ${ms}ms before ${todo.value.task} on ${todo.value.session}`,
            );
            await this.wait(ms);
          }

          return Some(todo.value);
        })()),
    };
  }

  private wait(ms: number) {
    return ms > 0 ? this.sleep(ms) : Promise.resolve();
  }
}
