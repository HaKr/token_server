import { Err, Ok, Result } from "./result.ts";

export class Option<T> {
  val: T | undefined;
  isnone = true;

  static from<U>(v: U | undefined): Option<U> {
    return v === undefined ? Option.None<U>() : Some(v);
  }

  is_none() {
    return this.isnone;
  }

  is_some() {
    return !this.isnone;
  }

  insert(v: T): Option<T> {
    this.val = v;
    this.isnone = false;

    return this;
  }

  clear(): Option<T> {
    this.val = undefined;
    this.isnone = true;
    return this;
  }

  and_then<U>(action: (val: T) => Option<U>): Option<U> {
    return this.isnone ? Option.None<U>() : action(this.val!);
  }

  or_else(action: () => Option<T>): Option<T> {
    return this.isnone ? action() : this;
  }

  async async_or_else(action: () => Promise<Option<T>>): Promise<Option<T>> {
    return this.isnone ? await action() : this;
  }

  async_map_or_else<U>(
    def: () => Promise<U>,
    action: (v: T) => Promise<U>,
  ): Promise<U> {
    return this.isnone ? def() : action(this.val!);
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
    return this.isnone ? Option.None<U>() : Some(mapper(this.val!));
  }

  async async_map<U>(mapper: (v: T) => Promise<U>): Promise<Option<U>> {
    return this.isnone ? Option.None<U>() : Some(await mapper(this.val!));
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

  toString() {
    return this.isnone ? "None" : `Some(${this.val})`;
  }

  static new_some<U>(v: U): Option<U> {
    return new Option<U>().with_val(v);
  }

  static None<U>(): Option<U> {
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
