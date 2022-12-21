import { Err, Ok, Option, Result } from "./deps.ts";

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
  static parse<O extends Options>(
    def: O,
    showHelp: (opts: O) => void,
  ): Result<O, ParseError> {
    const last = Option.none<string>();
    let result = Ok<boolean, ParseError>(true);
    (def as Options).help = false;
    const iter = Deno.args[Symbol.iterator]();
    for (
      let next_arg = iter.next();
      result.isOk() && !next_arg.done;
      next_arg = iter.next()
    ) {
      const arg = next_arg.value;
      if (arg.startsWith("--")) {
        const name = arg.slice(2).replace("-", "_");
        result = last.mapOrElse(
          () => {
            if ((def as Options)[name] == undefined) {
              return Err(new UnknownArgument(name));
            }
            if (typeof (def as Options)[name] == "boolean") {
              (def as Options)[name] = true;
            } else {
              last.insert(name);
            }
            return Ok(true);
          },
          (arg) => Err<boolean, ParseError>(new MissingA_ValueFor(arg)),
        );
      } else {
        result = last.okOrElse(() => new DanglingValue(arg))
          .andThen<true>((name) => {
            if (typeof (def as Options)[name] == "boolean") {
              return Err(new MustHaveNoValue(name));
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

    return result.andThen<O>(() => {
      return last.mapOrElse(
        () => Ok<O, ParseError>(def),
        (name) => Err<O, ParseError>(new NotA_Switch(name)),
      ).andThen((opts) => {
        const res = opts as { help?: boolean };

        const needHelp = res.help;
        delete res.help;
        if (needHelp) {
          showHelp(opts);
          return Err(new HelpWasDisplayed());
        }
        return Ok(res as O);
      });
    }).mapErr((err) => {
      if (err instanceof HelpWasDisplayed) return err;

      console.error("\n\n", err.toString(), "\n");
      showHelp(def);
      return new HelpWasDisplayed() as ParseError;
    });
  }
}

export class DanglingValue extends ParseError {}
export class UnknownArgument extends ParseError {}
export class NotA_Switch extends ParseError {}
export class MustHaveNoValue extends ParseError {}
export class MissingA_ValueFor extends ParseError {}
export class CouldNotUnwrap extends ParseError {}
export class HelpWasDisplayed extends ParseError {
  constructor() {
    super("");
  }
}
