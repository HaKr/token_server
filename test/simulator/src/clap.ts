import { Err, None, Ok, Options, ParseError, Result } from "./deps.ts";

export class CommandLine {
  static parse<O extends Options>(
    optionDefaults: O,
    showHelp: (opts: O) => void,
  ): Result<O, ParseError> {
    const lastArgumentName = None<string>();
    let result: Result<Options, ParseError> = Ok(optionDefaults as Options);
    (optionDefaults as Options).help = false;
    const argumentIterator = Deno.args[Symbol.iterator]();
    for (
      let nextArgument = argumentIterator.next();
      result.isOk() && !nextArgument.done;
      nextArgument = argumentIterator.next()
    ) {
      const argument = nextArgument.value;
      if (argument.startsWith("--")) {
        const name = argument.slice(2).replace("-", "_");
        lastArgumentName.mapOrElse(
          () => {
            if ((optionDefaults as Options)[name] == undefined) {
              result = Err<O, ParseError>(new UnknownArgument(name));
            }
            if (typeof (optionDefaults as Options)[name] == "boolean") {
              (optionDefaults as Options)[name] = true;
            } else {
              lastArgumentName.insert(name);
            }
          },
          (arg) => {
            result = Err<O, ParseError>(new MissingAValueFor(arg));
          },
        );
      } else { // does not start with --
        lastArgumentName.mapOrElse(
          () => {
            result = Err(new DanglingValue(argument));
          },
          (name) => {
            if (typeof (optionDefaults as Options)[name] == "boolean") {
              result = Err(new MustHaveNoValue(name));
            } else {
              (optionDefaults as Options)[name] =
                typeof (optionDefaults as Options)[name] == "number"
                  ? Number.parseInt(argument, 10) || 0
                  : argument;
              lastArgumentName.take();
            }
          },
        );
      }
    }

    return result.andThen((opts) =>
      lastArgumentName.mapOrElse(
        (): Result<O, ParseError> => {
          const res = opts as { help?: boolean };
          const args = opts as O;

          const needHelp = res.help;
          delete res.help;
          if (needHelp) {
            showHelp(args);
            return Err(new HelpWasDisplayed());
          }
          return Ok(args) as unknown as Result<O, ParseError>;
        },
        (arg): Result<O, ParseError> => Err(new NotASwitch(arg)),
      )
    )
      .mapErr((err): ParseError => {
        if (err instanceof HelpWasDisplayed) return err;

        console.error("\n\n", err.toString(), "\n");
        showHelp(optionDefaults);
        return new HelpWasDisplayed();
      });
  }
}

export class DanglingValue extends ParseError {}
export class UnknownArgument extends ParseError {}
export class NotASwitch extends ParseError {}
export class MustHaveNoValue extends ParseError {}
export class MissingAValueFor extends ParseError {}
export class CouldNotUnwrap extends ParseError {}
export class HelpWasDisplayed extends ParseError {
  constructor() {
    super("");
  }
}
