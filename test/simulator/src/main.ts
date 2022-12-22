import { CommandLine, HelpWasDisplayed } from "./clap.ts";
import { None } from "./deps.ts";
import { Simulator } from "./simulator.ts";

type Options = {
  name: string;
  include_errors: boolean;
  random_wait: number;
  shutdown_after: number;
};

const options: Options = {
  name: "sim",
  include_errors: false,
  random_wait: 0,
  shutdown_after: 0,
};

CommandLine.parse(options, showHelp)
  .mapOrElse(
    (err) => {
      if (!(err instanceof HelpWasDisplayed)) console.error(err.toString());
    },
    async (options) => {
      Simulator.LOGGER.debug(options);

      const simulator = new Simulator(
        options.name,
        options.include_errors,
        options.random_wait,
      );

      const handle = None<number>();
      if (options.shutdown_after > 0) {
        handle.insert(setTimeout(
          () => simulator.shutdownServer(),
          options.shutdown_after * 1000,
        ));
      }

      await simulator.run()
        .then(
          (res) => {
            res.mapOrElse(
              (err) => Simulator.LOGGER.error(err),
              () =>
                Simulator.LOGGER.info(`Simulation ${options.name} finished`),
            );
          },
          (err) => Simulator.LOGGER.error(err),
        );

      handle.map(clearTimeout);
    },
  );

function showHelp(opts: Options) {
  console.log(`  
Usage: LOG=tokenclient=[LEVEL],simulator=[LEVEL],scheduler=[LEVEL] deno run --allow-net --allow-env src/main.ts [OPTIONS]

LEVEL = trace | debug | info | warn | error | off
  Verbosity of the specified module

Options:
  --help
      Show this help

  --name STRING
      Optional name for the simulation, default: sim (${opts.name})

  --include-error 
      Add this argument to generate some errors (${opts.include_errors})

  --random-wait INTEGER
      Pass a value to specify the maximum number of seconds to wait between simulation tasks.
      When no positive integer is specified, all tasks are performed without any added delay.
      (${opts.random_wait > 0 ? `< ${opts.random_wait}s` : "no delay"})

  --shutdown-after INTEGER
      Pass a value to specify the number of seconds after which to shutdown the token server.
      When no positive integer is specified, the token server will remain active.
      (${opts.shutdown_after > 0 ? `${opts.shutdown_after}s` : "no shutdown"})
`);
}
