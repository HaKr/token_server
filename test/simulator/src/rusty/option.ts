import { Err, Ok, Result } from "./result.ts";
import type { FutureResult, ResultOrFuture } from "./result.ts";

type PossibleFuture<T> = T | Promise<T>;
type OptionOrFuture<T> = PossibleFuture<Option<T>>;
type FutureOption<T> = Promise<Option<T>>;

export class Option<T> {
  val: T | undefined;
  isnone = true;

  static from<U>(v: U | undefined): Option<U> {
    return v === undefined ? Option.none<U>() : Some(v);
  }

  isNone() {
    return this.isnone;
  }

  isSome() {
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

  /**
   * Returns None if the option is None, otherwise calls f with the wrapped value and returns the result.
   *
   * Some languages call this operation flatmap.
   */
  andThen<U>(action: (some: T) => FutureOption<U>): FutureOption<U>;
  andThen<U>(action: (some: T) => Option<U>): Option<U>;
  andThen<U>(
    action: (some: T) => OptionOrFuture<U>,
  ): OptionOrFuture<U> {
    if (this.isnone) return this as unknown as Option<U>;
    const res = action(this.val!);
    return res instanceof Promise
      ? res.then(Option.some) as FutureOption<U>
      : res;
  }

  /**
   * Returns the option if it contains a value, otherwise calls action and returns the result.
   */
  orElse(action: () => FutureOption<T>): FutureOption<T>;
  orElse(action: () => Option<T>): Option<T>;
  orElse(
    action: () => OptionOrFuture<T>,
  ): OptionOrFuture<T> {
    if (!this.isnone) return this;
    const alt = action();
    return alt instanceof Promise
      ? alt.then(Option.some) as FutureOption<T>
      : this;
  }

  /**
   * Transforms the Option<T> into a Result<T, E>, mapping Some(v) to Ok(v) and None to Err(E).
   */
  okOr<E>(err: E): Result<T, E> {
    return (this.isnone ? Err(err) : Ok(this.val!)) as Result<T, E>;
  }

  /**
   * Transforms the Option<T> into a Result<T, E>, mapping Some(v) to Ok(v) and None to Err(mapper()).
   */
  okOrElse<E>(mapper: () => Promise<E>): ResultOrFuture<T, E>;
  okOrElse<E>(mapper: () => E): Result<T, E>;
  okOrElse<E>(mapper: () => E | Promise<E>): ResultOrFuture<T, E> {
    if (!this.isnone) return Ok(this.val!) as Result<T, E>;
    const err = mapper();

    return err instanceof Promise
      ? err.then(Err) as FutureResult<T, E>
      : Err(err);
  }

  /**
   * Maps an Option<T> to Option<U> by applying a function to a contained value.
   */
  map<U>(action: (some: T) => Promise<U>): FutureOption<U>;
  map<U>(action: (some: T) => U): Option<U>;
  map<U>(
    action: (some: T) => U | Promise<U>,
  ): OptionOrFuture<U> {
    if (this.isnone) return this as unknown as Option<U>;

    const alt = action(this.val!);

    return alt instanceof Promise ? alt.then(Option.some) : Option.some(alt);
  }

  mapOr<U>(def: U, mapper: (v: T) => U): Option<U> {
    return this.isnone ? Some(def) as Option<U> : Some(mapper(this.val!));
  }

  /**
   * Computes a default function result (if none), or applies a different function to the contained value (if any).
   */
  mapOrElse<U>(
    def: () => U,
    mapper: (v: T) => Promise<U>,
  ): U | Promise<U>;
  mapOrElse<U>(
    def: () => Promise<U>,
    mapper: (v: T) => U,
  ): U | Promise<U>;
  mapOrElse<U>(
    def: () => Promise<U>,
    mapper: (v: T) => Promise<U>,
  ): Promise<U>;
  mapOrElse<U>(def: () => U, mapper: (v: T) => U): U;
  mapOrElse<U>(
    def: () => U | Promise<U>,
    mapper: (v: T) => U | Promise<U>,
  ): U | Promise<U> {
    return this.isnone ? def() : mapper(this.val!);
  }

  unwrapOr(alt: T): T {
    return this.isnone ? alt : this.val!;
  }

  toString() {
    return this.isnone ? "None" : `Some(${this.val})`;
  }

  static some<U>(v: U): Option<U> {
    return new Option<U>().with_val(v);
  }

  static none<U>(): Option<U> {
    return new Option<U>();
  }

  private with_val(v: T) {
    this.val = v;
    this.isnone = false;

    return this;
  }
}

export function Some<T>(x: T): Option<T> {
  return Option.some(x);
}

export function None<T>(): Option<T> {
  return Option.none();
}
