enum Level {
  NONE,
  ERROR,
  WARNING,
  INFO,
  DEBUG,
  TRACE,
}

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

  private constructor(private level: Level) {}

  public static configure(levels: string) {
    levels.split(",").forEach((assignment) => {
      const [name, levelName] = assignment.split("=");
      Logging.levelMap[name.toLowerCase()] = Logging.levelFromName[levelName] ||
        Logging.level;
    });
  }

  public static for(module: string): Logging {
    const mappedLevel = Logging.levelMap[module.toLowerCase()];

    return new Logging(mappedLevel != undefined ? mappedLevel : Logging.level);
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
    if (this.level >= level) {
      console.log(...args);
    }
    return args;
  }
}

Logging.configure(Deno.env.get("LOG") || "");
