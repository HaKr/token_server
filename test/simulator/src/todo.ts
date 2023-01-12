import { Session } from "./session.ts";
import { TaskName } from "./tasks.ts";

export class ToDo {
  constructor(
    public session: Session,
    public task: TaskName,
    public when: number,
  ) {}

  shouldExecute() {
    const hasToken = this.session.hasToken();
    return this.when < 1 ? !hasToken : hasToken;
  }

  toString() {
    return `${this.task} on ${this.session}`;
  }
}
