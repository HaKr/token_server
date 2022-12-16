import { Err, Ok, Result } from "./result.ts";

export class Option<T> {
  val: T | undefined;
  isnone = true;

  static from<U>(v: U | undefined): Option<U> {
    return v === undefined ? None as Option<U> : Some(v);
  }

  is_none() {
    return this.isnone;
  }

  is_some() {
    return !this.isnone;
  }

  ok_or<E>(err: E): Result<T, E> {
    return (this.isnone ? Err(err) : Ok(this.val!)) as unknown as Result<T, E>;
  }

  ok_or_else<E>(mapper: () => E): Result<T, E> {
    return (this.isnone ? Err(mapper()) : Ok(this.val!)) as unknown as Result<
      T,
      E
    >;
  }

  map<U>(mapper: (v: T) => U): Option<U> {
    return this.isnone ? None as Option<U> : Some(mapper(this.val!));
  }

  map_or<U>(def: U, mapper: (v: T) => U): Option<U> {
    return this.isnone ? Some(def) as Option<U> : Some(mapper(this.val!));
  }

  async async_map_or<U>(
    def: U,
    mapper: (v: T) => Promise<U>,
  ): Promise<Option<U>> {
    return this.isnone ? Some(def) as Option<U> : Some(await mapper(this.val!));
  }

  map_or_else<U>(def: () => U, mapper: (v: T) => U): Option<U> {
    return this.isnone ? Some(def()) as Option<U> : Some(mapper(this.val!));
  }

  unwrap_or(alt: T): T {
    return this.isnone ? alt : this.val!;
  }

  static new_some<U>(v: U): Option<U> {
    return new Option<U>().with_val(v);
  }

  static new_none<U>(): Option<U> {
    return new Option<U>();
  }

  private with_val(v: T) {
    this.val = v;
    this.isnone = false;

    return this;
  }
}

export function Some<T>(x: T): Option<T> {
  return Option.new_some(x);
}

export const None = Option.new_none();
