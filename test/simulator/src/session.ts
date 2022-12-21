import { Meta } from "./api.ts";
import { Option, Result } from "./deps.ts";

export class Session {
  static count = 1;

  token = Option.none<string>();
  public index = Session.count++;

  constructor(private meta_: Meta) {}

  create(t: string) {
    this.token.insert(t);
  }

  get meta() {
    return this.meta_;
  }

  clear_token() {
    this.token.clear();
  }

  has_token() {
    return this.token.isSome();
  }

  update(token: string, meta: Meta) {
    this.token.insert(token);
    this.meta_ = meta;
  }

  token_or_else<F>(err_mapper: () => F): Result<string, F> {
    return this.token.okOrElse(err_mapper);
  }

  toString() {
    return (`[(${
      this.token.mapOr(" no  ", (_) => "token").unwrapOr(
        "impossible",
      )
    }) ` +
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
