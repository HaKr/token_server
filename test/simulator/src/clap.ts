import { Option } from "./std/option.ts";
import { Err, Ok, Result } from "./std/result.ts";

type Options = { [key: string]: string | number | boolean };

export abstract class ParseError {
  constructor(public arg: string) {}

  toString() {
    return `${
      this.constructor.name.replace(/(\w)([A-Z])/g, (_sub, ...args) => {
        const [a, b] = args as string[];
        return `${a == "_" ? "" : a} ${b.toLowerCase()}`;
      })
    }: ${this.arg}`;
  }
}

export class CommandLine {
  static parse<O extends Options>(def: O): Result<O, ParseError> {
    const last = Option.None<string>();
    let result = Ok<boolean, ParseError>(true);
    const iter = Deno.args[Symbol.iterator]();
    for (
      let next_arg = iter.next();
      result.is_ok() && !next_arg.done;
      next_arg = iter.next()
    ) {
      const arg = next_arg.value;
      if (arg.startsWith("--")) {
        last.map((name) => {
          if (typeof (def as Options)[name] == "boolean") {
            (def as Options)[name] = true;
            last.clear();
          } else {
            result = Err(new NotA_Switch(name));
          }
        });
        const name = arg.slice(2).replace("-", "_");
        if ((def as Options)[name] == undefined) {
          result = Err(new UnknownArgument(name));
        } else {
          last.insert(name);
        }
      } else {
        result = last.ok_or_else(() => new DanglingValue(arg))
          .and_then((name) => {
            if (typeof (def as Options)[name] == "boolean") {
              return Err(new HasNoValue(name));
            } else {
              (def as Options)[name] = typeof (def as Options)[name] == "number"
                ? Number.parseInt(arg, 10) || 0
                : arg;
              last.clear();
              return Ok(true);
            }
          });
      }
    }

    return result.and_then(() => {
      return last.map_or_else<Result<O, ParseError>>(
        () => Ok(def),
        (name) => {
          if (typeof (def as Options)[name] == "boolean") {
            (def as Options)[name] = true;
            return Ok<O, ParseError>(def);
          } else {
            return Err<O, ParseError>(new NotA_Switch(name));
          }
        },
      ).unwrap_or(Err(new CouldNotUnwrap("last option value")));
    });
  }
}

export class DanglingValue extends ParseError {}
export class UnknownArgument extends ParseError {}
export class NotA_Switch extends ParseError {}
export class HasNoValue extends ParseError {}
export class CouldNotUnwrap extends ParseError {}
