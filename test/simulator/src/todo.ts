import { Session } from "./session.ts";
import { Task } from "./tasks.ts";

export class ToDo {
  constructor(
    public session: Session,
    public task: Task,
    public when: number,
  ) {}

  shouldExecute() {
    const hasToken = this.session.has_token();
    return this.when < 1 ? !hasToken : hasToken;
  }
}
