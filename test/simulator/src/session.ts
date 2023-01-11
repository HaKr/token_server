import { Meta } from "./api.ts";
import { None, Result } from "./deps.ts";

export class Session {
  static count = 1;

  token = None<string>();
  public index = Session.count++;

  constructor(private meta_: Meta) {}

  create(t: string) {
    this.token.insert(t);
  }

  get meta() {
    return this.meta_;
  }

  clearToken() {
    this.token.take();
  }

  hasToken() {
    return this.token.isSome();
  }

  update(token: string, meta: Meta) {
    this.token.insert(token);
    this.meta_ = meta;
  }

  tokenOrElse<F>(err_mapper: () => F): Result<string, F> {
    return this.token.okOrElse(err_mapper);
  }

  toString() {
    return (`[(${this.token.mapOrElse(() => " no  ", (_) => "token")}) ` +
      (this.meta_ === null
        ? "NIL"
        : `${
          this.meta_.lastName
            ? this.meta_.lastName
            : this.meta_.year
            ? this.meta_.year
            : ""
        }`) +
      `${
        (this.meta_ !== null && (this.meta_.lastName || this.meta_.year)) &&
          this.meta_.updatedAt
          ? `, ${this.meta_.updatedAt}`
          : ""
      }`)
      .padEnd(38) + "]";
  }
}
