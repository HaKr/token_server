import { Meta } from "./api.ts";
import { Option } from "./std/option.ts";
import { Result } from "./std/result.ts";

export class Session {
  static count = 1;

  token = Option.None<string>();
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
    return this.token.is_some();
  }

  update(token: string, meta: Meta) {
    this.token.insert(token);
    this.meta_ = meta;
  }

  token_or_else<F>(err_mapper: () => F): Result<string, F> {
    return this.token.ok_or_else(err_mapper);
  }

  toString() {
    return (`[(${
      this.token.map_or(" no  ", (_) => "token").unwrap_or(
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
