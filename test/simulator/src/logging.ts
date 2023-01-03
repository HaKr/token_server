import { Option, optionFrom } from "./deps.ts";

enum Level {
  NONE,
  ERROR,
  WARNING,
  INFO,
  DEBUG,
  TRACE,
}

const levelLabel = {
  [Level.NONE]: "",
  [Level.ERROR]: "ERR",
  [Level.WARNING]: "WRN",
  [Level.INFO]: "INF",
  [Level.DEBUG]: "DBG",
  [Level.TRACE]: "TRC",
};

export class Logging {
  static levelFromName: { [key: string]: Level } = {
    "off": Level.NONE,
    "error": Level.ERROR,
    "warn": Level.WARNING,
    "info": Level.INFO,
    "debug": Level.DEBUG,
    "trace": Level.TRACE,
  };
  static level = Level.INFO;
  static levelMap: { [key: string]: Level } = {};

  private constructor(private module: string) {}

  public static configure(levels: string) {
    levels.split(",").forEach((assignment) => {
      const [name, levelName] = levelDefinition(assignment);
      levelName.mapOrElse(
        () => Logging.level = Logging.levelFromName[name] || Level.INFO,
        (levelName) =>
          Logging.levelMap[name.toLowerCase()] =
            Logging.levelFromName[levelName] ||
            Logging.level,
      );
    });
  }

  public static for(module: string): Logging {
    return new Logging(module);
  }

  static levelFor(module: string) {
    return optionFrom(Logging.levelMap[module.toLowerCase()]).unwrapOrElse(() =>
      Logging.level
    );
  }

  error(...args: unknown[]) {
    return this.log(Level.ERROR, ...args);
  }

  warning(...args: unknown[]) {
    return this.log(Level.WARNING, ...args);
  }

  info(...args: unknown[]) {
    return this.log(Level.INFO, ...args);
  }

  debug(...args: unknown[]) {
    return this.log(Level.DEBUG, ...args);
  }

  trace(...args: unknown[]) {
    return this.log(Level.TRACE, ...args);
  }

  private log(level: Level, ...args: unknown[]) {
    if (Logging.levelFor(this.module) >= level) {
      console.log(`${levelLabel[level]}-[${this.module.padEnd(11)}]`, ...args);
    }
    return args;
  }
}

Logging.configure(Deno.env.get("LOG") || "");

function levelDefinition(assignment: string): [string, Option<string>] {
  const [moduleName, level] = assignment.split("=");
  return [moduleName, optionFrom(level)];
}
